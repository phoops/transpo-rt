use std::u64;

use super::open_api::make_param;
use crate::datasets::{Connection, Dataset, RealTimeConnection, UpdatedTimetable};
use crate::extractors::RealTimeDatasetWrapper;
use crate::routes::estimated_timetable;
use crate::siri_lite::{self, service_delivery as model, SiriResponse};
use crate::utils;
use actix_web::{error, web};
use openapi_schema::OpenapiSchema;
use transit_model::collection::Idx;
use transit_model::objects::StopPoint;

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
        StopPointRef: format!("IT:ITC1:ScheduledStopPoint:busATS:{}", stop.id.clone()), //TODO: hardcoded prefix
        visit_number: None, //TODO: find value
        order: connection.sequence as u16,
        stop_point_name: stop.name.clone(),
        aimed_arrival_time: Some(siri_lite::DateTime(connection.arr_time)),
        aimed_departure_time: Some(siri_lite::DateTime(connection.dep_time)),
        expected_arrival_time: updated_connection
            .and_then(|c| c.arr_time)
            .map(siri_lite::DateTime),
        expected_departure_time: updated_connection
            .and_then(|c| c.dep_time)
            .map(siri_lite::DateTime),
    };

    model::EstimatedVehicleJourney {
            line_ref: format!("IT:ITC1:Line:busATS:{}", line_ref.clone()), //TODO: hardcoded prefix
            direction_ref: None, //TODO: find value
            journey_pattern_ref: None, //TODO: find value
            published_line_name: None, //TODO: find value
            operator_ref: model::ServiceInfoGroup { 
                operator_ref: Some(format!("IT:ITC1:Operator:12345678911:busATS:{}", operator_ref.unwrap_or_default())) 
            }, //TODO: hardcoded prefix
            veichle_ref: None, //TODO: find value
            estimated_calls: vec![call]
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
        response_time_stamp: chrono::Local::now().to_rfc3339(), //TODO: is this value correct?
        request_message_ref: None,
        subscriber_ref: "NAP".to_string(), //TODO: hardcoded value
        subscription_ref: "0001".to_string(), //TODO: hardcoded value
        estimated_journey_version_frame: model::EstimatedJourneyVersionFrame {
            recorded_at_time: chrono::Local::now().to_rfc3339(), //TODO: is this value correct?
            estimated_vehicle_journey: estimated_timetable,
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
) -> actix_web::Result<siri_lite::SiriResponse> {
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

    Ok(siri_lite::SiriResponse {
        siri: siri_lite::Siri {
            service_delivery: Some(model::ServiceDelivery {
                producer_ref: Some("RAP_Toscana".to_string()), // TODO: hardcoded value
                estimated_timetable_delivery: create_estimated_timetable(
                    stop_idx,
                    &data,
                    updated_timetable,
                    &request,
                ),
                response_message_identifier:  Some("0001".to_string()),   //TODO: hardcoded prefix
                ..Default::default()
            }),
            ..Default::default()
        },
    })
}

pub async fn estimated_timetable_query(
    web::Query(query): web::Query<Params>,
    rt_dataset_wrapper: RealTimeDatasetWrapper,
) -> actix_web::Result<web::Json<SiriResponse>> {
    Ok(web::Json(estimated_timetable(query, rt_dataset_wrapper)?))
}
