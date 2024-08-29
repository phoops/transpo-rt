use std::u64;

use super::open_api::make_param;
use crate::datasets::{Connection, Dataset, RealTimeConnection, UpdatedTimetable};
use crate::extractors::RealTimeDatasetWrapper;
use crate::routes::estimated_timetable;
use crate::siri_lite::{self, service_delivery as model, SiriResponse};
use crate::utils;
use actix_web::{error, web, HttpResponse};
use openapi_schema::OpenapiSchema;
use transit_model::collection::Idx;
use transit_model::objects::StopPoint;
use quick_xml::se::to_string;

#[derive(Debug, Deserialize, PartialEq, Eq, OpenapiSchema)]
enum DataFreshness {
    RealTime,
    Scheduled,
}

impl Default for DataFreshness {
    fn default() -> Self {
        DataFreshness::RealTime
    }
}

fn default_stop_visits() -> u64 {
    u64::MAX
}

fn default_only_realtime() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Params {
    _requestor_ref: Option<String>,
    /// Id of the stop_point on which we want the next departures
    monitoring_ref: Option<String>,
    /// Filter only realtime data
    #[serde(default = "default_only_realtime")]
    only_realtime: bool,
    /// Filter the departures of the given OperatorRef
    operator_ref: Option<String>,
    /// Filter the departures of the given line's id
    line_ref: Option<String>,
    _destination_ref: Option<String>,
    /// start_time is the datetime from which we want the next departures
    /// The default is the current time of the query
    start_time: Option<siri_lite::DateTime>,

    /// ISO 8601 duration used to filter the departures/arrivals
    /// within the period [start_time, start_time + duration]
    /// example format: 'PT10H' for a 10h duration
    preview_interval: Option<utils::Duration>,
    /// the data_freshness is used to control whether we want realtime data or only base schedule data
    #[serde(default = "DataFreshness::default")]
    data_freshness: DataFreshness,
    #[serde(default = "default_stop_visits")]
    maximum_stop_visits: u64,
}

impl Params {
    // TODO: generate this via derive macro
    pub fn openapi_description(spec: &mut openapi::v3_0::Spec) -> Vec<openapi::v3_0::Parameter> {
        vec![
            make_param::<String>(spec, "MonitoringRef", false),
            make_param::<bool>(spec, "OnlyRealtime", false),
            make_param::<String>(spec, "OperatorRef", false),
            make_param::<String>(spec, "LineRef", false),
            make_param::<siri_lite::DateTime>(spec, "StartTime", false),
            make_param::<DataFreshness>(spec, "DataFreshness", false),
            make_param::<utils::Duration>(spec, "PreviewInterval", false),
            make_param::<u16>(spec, "MaximumStopVisits", false),
        ]
    }
}

fn create_estimated_timetable_visit(
    data: &Dataset,
    connection: &Connection,
    updated_connection: Option<&RealTimeConnection>,
) -> siri_lite::service_delivery::EstimatedVehicleJourney {
    let stop = &data.ntm.stop_points[connection.stop_point_idx];
    let vj = &data.ntm.vehicle_journeys[connection.dated_vj.vj_idx];
    let route = &data.ntm.routes.get(&vj.route_id);
    // we consider that the siri's operator in transmodel's company
    let operator_ref = data
        .ntm
        .get_corresponding_from_idx(connection.dated_vj.vj_idx)
        .into_iter()
        .next()
        .map(|idx| data.ntm.companies[idx].id.clone());
    let line_ref = route
        .map(|r| r.line_id.clone())
        .unwrap_or_else(|| "".to_owned());
    let update_time = updated_connection
        .map(|c| c.update_time)
        // if we have no realtime data, we consider the update time to be the time of the base schedule loading
        // (it's not that great, but we don't have something better)
        .unwrap_or_else(|| data.loaded_at);
    let call = model::EstimatedCall {
        StopPointRef: model::StopPointRefWrapper{ StopPointRef: format!("IT:ITC1:ScheduledStopPoint:busATS:{}", stop.id.clone())}, //TODO: hardcoded prefix
        VisitNumber: None, //TODO: find value
        Order: model::OrderWrapper{ Order: connection.sequence as u16},
        StopPointName: model::StopPointNameWrapper{ StopPointName: stop.name.clone()},
        AimedArrivalTime: Some( model::AimedArrivalTimeWrapper{ AimedArrivalTime: siri_lite::DateTime(connection.arr_time)}),
        ExpectedArrivalTime: updated_connection
            .and_then(|c| c.arr_time)
            .map(|time| model::ExpectedArrivalTimeWrapper { ExpectedArrivalTime: siri_lite::DateTime(time) }),
        AimedDepartureTime: Some( model::AimedDepartureTimeWrapper{ AimedDepartureTime: siri_lite::DateTime(connection.dep_time)}),
        ExpectedDepartureTime: updated_connection
            .and_then(|c| c.dep_time)
            .map(|time| model::ExpectedDepartureTimeWrapper { ExpectedDepartureTime: siri_lite::DateTime(time) })
        };

    model::EstimatedVehicleJourney {
        LineRef: model::LineRefWrapper{ LineRef: format!("IT:ITC1:Line:busATS:{}", line_ref.clone())}, //TODO: hardcoded prefix
        DirectionRef: Some(model::DirectionRefWrapper{ DirectionRef: "inbound".to_string()}), //TODO: find value
        JourneyPatternRef: None, //TODO: find value
        PublishedLineName: None, //TODO: find value
        FramedVehicleJourneyRef: model::FramedVehicleJourneyRef {
            DataFrameRef: Some(model::DataFrameRefWrapper{ DataFrameRef: connection.arr_time.date().to_string() }), 
            DatedVehicleJourneyRef: Some(model::DatedVehicleJourneyRefWrapper{ DatedVehicleJourneyRef: format!("x")}), //TODO: hardcoded values
        },
        OperatorRef: model::ServiceInfoGroup { 
            OperatorRef: Some(model::OperatorRefWrapper{ OperatorRef: format!("IT::Operator:02194050486:{}", operator_ref.unwrap_or_default())}) 
        }, //TODO: hardcoded prefix
        VehicleRef: None, //TODO: find value
        EstimatedCalls: call,
    }
}

fn get_line_ref<'a>(cnx: &Connection, model: &'a transit_model::Model) -> Option<&'a str> {
    let vj = &model.vehicle_journeys[cnx.dated_vj.vj_idx];
    model.routes.get(&vj.route_id).map(|r| r.line_id.as_str())
}

fn get_operator_ref<'a>(cnx: &Connection, model: &'a transit_model::Model) -> Option<&'a str> {
    model
        .get_corresponding_from_idx(cnx.dated_vj.vj_idx)
        .into_iter()
        .next()
        .map(|idx| model.companies[idx].id.as_str())
}

fn is_in_interval(
    cnx: &Connection,
    start_time: chrono::NaiveDateTime,
    duration: &Option<utils::Duration>,
) -> bool {
    duration
        .as_ref()
        .map(|duration| {
            let limit = start_time + **duration;
            cnx.dep_time <= limit || cnx.arr_time <= limit
        })
        .unwrap_or(true)
}

fn create_estimated_timetable(
    stop_idx: std::option::Option<Idx<StopPoint>>,
    data: &Dataset,
    updated_timetable: &UpdatedTimetable,
    request: &Params,
) -> Vec<model::EstimatedTimetableDelivery> {
    // if we want to datetime in the query, we get the current_time (in the timezone of the dataset)
    let requested_start_time = request.start_time.as_ref().map(|d| d.0).unwrap_or_else(|| {
        chrono::Utc::now()
            .with_timezone(&data.timezone)
            .naive_local()
    });
    let requested_line_ref = request.line_ref.as_deref();
    let requested_operator_ref = request.operator_ref.as_deref();

    let estimated_timetable = data
        .timetable
        .connections
        .iter()
        .enumerate()
        .skip_while(|(_, c)| c.dep_time < requested_start_time)
        .filter(|(_, c)| {
            stop_idx.is_none() || stop_idx == Some(c.stop_point_idx)
        })
        // filter on requested lines
        .filter(|(_, c)| {
            requested_line_ref.is_none() || requested_line_ref == get_line_ref(&c, &data.ntm)
        })
        // filter on requested operator
        .filter(|(_, c)| {
            requested_operator_ref.is_none() || requested_operator_ref == get_operator_ref(&c, &data.ntm)
        })
        .filter(|(_, c)| is_in_interval(&c, requested_start_time, &request.preview_interval))
        // filter if has realtime data has expected arrival time or expected departure time
        .filter(|(i, _c)| {
            let has_realtime_data = updated_timetable
                .realtime_connections
                .get(i)
                .map(|_c| _c.arr_time.is_some() || _c.dep_time.is_some())
                .unwrap_or(false);
            if request.only_realtime {
                has_realtime_data
            } else {
                true
            }
        })
        .map(|(idx, c)| {
            create_estimated_timetable_visit(
                data,
                c,
                match request.data_freshness {
                    DataFreshness::RealTime => updated_timetable.realtime_connections.get(&idx),
                    DataFreshness::Scheduled => None,
                },
            )
        })
        .take(request.maximum_stop_visits as usize)
        .collect::<Vec<model::EstimatedVehicleJourney>>();

    vec![model::EstimatedTimetableDelivery {
        ResponseTimestamp: model::ResponseTimeStampWrapper{ResponseTimestamp: chrono::Local::now().to_rfc3339()}, //TODO: is this value correct?
        RequestMessageRef: None,
        SubscriberRef: model::SubscriberRefWrapper{ SubscriberRef: "NAP".to_string()}, //TODO: hardcoded value
        SubscriptionRef: model::SubscriptionRefWrapper{ SubscriptionRef: "0001".to_string()}, //TODO: hardcoded value
        EstimatedJourneyVersionFrame: model::EstimatedJourneyVersionFrame {
            RecordedAtTime: model::RecordedAtTimeWrapper{ RecordedAtTime: chrono::Local::now().to_rfc3339()}, //TODO: is this value correct?
            EstimatedVehicleJourney: estimated_timetable,
        },
    }]
}

fn validate_params(request: &mut Params) -> actix_web::Result<()> {
    // we silently bound the maximum stop visits to 20
    //request.maximum_stop_visits = std::cmp::min(request.maximum_stop_visits, 20);
    
    Ok(())
}

fn estimated_timetable(
    mut request: Params,
    rt_dataset_wrapper: RealTimeDatasetWrapper,
) -> actix_web::Result<String> {
    let data = rt_dataset_wrapper.get_base_schedule_dataset()?;

    let updated_timetable = &rt_dataset_wrapper.updated_timetable;

    validate_params(&mut request)?;

    // TODO: hardcoded agency, remove when not necessary
    request.operator_ref = Some("1".to_string());

    // set stop_idx to a default value
    let mut stop_idx = None;
    if request.monitoring_ref.as_deref() != None {
        stop_idx = Some(data
            .ntm
            .stop_points
            .get_idx(request.monitoring_ref.as_deref().unwrap())
            .ok_or_else(|| {
                error::ErrorNotFound(format!(
                    "impossible to find stop: '{:?}'",
                    &request.monitoring_ref
                ))
            })?);
    }

    let service_delivery = Some(model::ServiceDelivery {
        ProducerRef: Some(model::ProducerRefWrapper{ ProducerRef: "RAP_Toscana".to_string()}), // TODO: hardcoded value
        EstimatedTimetableDelivery: create_estimated_timetable(
            stop_idx,
            &data,
            updated_timetable,
            &request,
        ),
        ResponseMessageIdentifier:  Some(model::ResponseMessageIdentifierWrapper{ ResponseMessageIdentifier: "0001".to_string()}),   //TODO: hardcoded prefix
        ResponseTimestamp: model::ResponseTimeStampWrapper{ ResponseTimestamp: chrono::Utc::now().to_rfc3339()},
        ..Default::default()
    });

    let service_delivery_xml = to_string(&service_delivery).unwrap();
    //TODO: FIX THIS replace
    let xml_with_root = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><Siri xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns=\"http://www.siri.org.uk/siri\" xsi:schemaLocation=\"http://www.siri.org.uk/siri ../xsd/siri.xsd\" version=\"2.1\"><ServiceDelivery>{}</ServiceDelivery></Siri>", service_delivery_xml
        .replace("<EstimatedCalls>", "<EstimatedCalls><EstimatedCall>")
        .replace("</EstimatedCalls>", "</EstimatedCall></EstimatedCalls>")
        .replace("</EstimatedCalls>", "</EstimatedCalls></EstimatedVehicleJourney><EstimatedVehicleJourney>")
        .replace("<EstimatedVehicleJourney></EstimatedVehicleJourney>", "")
    );

    Ok(xml_with_root)

}

pub async fn estimated_timetable_query(
    web::Query(query): web::Query<Params>,
    rt_dataset_wrapper: RealTimeDatasetWrapper,
) -> HttpResponse {
    let xml_string = estimated_timetable(query, rt_dataset_wrapper);
    let xml_string = xml_string.unwrap_or_default();
    HttpResponse::Ok()
        .content_type(actix_web::http::header::HeaderValue::from_static("text/plain"))
        .body(xml_string)
}
