use crate::siri_lite::general_message::GeneralMessageDelivery;
use crate::siri_lite::DateTime;
use openapi_schema::OpenapiSchema;

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "camelCase")]
pub enum ArrivalStatus {
    OnTime,
    Early,
    Delayed,
    Cancelled,
    Missed,
    Arrived,
    NotExpected,
    NoReport,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoredCall {
    pub order: u16,
    pub stop_point_name: String,
    /// true if the vehicle is at the stop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// Destination on the headsign of the vehicle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_display: Option<String>,
    /// Scheduled arrival time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime>,
    /// Scheduled departure time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime>,
    /// Estimated arrival time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime>,
    /// Estimated departure time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime>,
    /// Status on the arrival at the stop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrival_status: Option<ArrivalStatus>,
}


#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedVehicleJourney {
    pub line_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_line_name: Option<String>,
    #[serde(flatten)]
    pub operator_ref: ServiceInfoGroup,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub veichle_ref: Option<String>,
    /// Id of the journey pattern
    pub estimated_calls: Vec<EstimatedCall>,
}


#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedCall {
    pub StopPointRef: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<String>,
    pub order: u16,
    pub stop_point_name: String,
    /// Scheduled arrival time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime>,
    /// Scheduled departure time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime>,
    /// Estimated arrival time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime>,
    /// Estimated departure time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfoGroup {
    /// Id of the operator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<String>,
    /* TODO find the right documentation for the type of this
    /// Specific features available in the vehicle
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<String>,
    */
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoredVehicleJourney {
    #[serde(flatten)]
    pub service_info: ServiceInfoGroup,
    /// Id of the journey pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<String>,
    // pub onward_calls: Option<OnwardCall>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoredStopVisit {
    /// Id of the stop point
    pub monitoring_ref: String,
    /// Datetime of the information update
    pub recorded_at_time: chrono::DateTime<chrono::Utc>,
    /// Id of the couple Stop / VehicleJourney
    pub item_identifier: String,
    pub monitored_vehicle_journey: MonitoredVehicleJourney,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedTimetableVisit {
    /// Id of the couple Stop / VehicleJourney
    //pub item_identifier: String,
    pub monitored_vehicle_journey: EstimatedVehicleJourney,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct StopMonitoringDelivery {
    /// Version of the siri's response
    pub version: String,
    /// Datetime of the response's production
    pub response_time_stamp: String,
    /// Id of the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<String>, // Note: this is mandatory for idf profil
    /// Status of the response, true if the response has been correctly treated, false otherwise
    pub status: bool,
    pub monitored_stop_visit: Vec<MonitoredStopVisit>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedTimetableDelivery {
    /// Datetime of the response's production
    pub response_time_stamp: String,
    /// Id of the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<String>, // Note: this is mandatory for idf profil
    pub subscriber_ref: String,
    pub subscription_ref: String,
    pub estimated_journey_version_frame: EstimatedJourneyVersionFrame,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedJourneyVersionFrame {
    pub recorded_at_time: String,
    pub estimated_vehicle_journey: Vec<EstimatedVehicleJourney>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceDelivery {
    #[serde(flatten)]
    pub common: crate::siri_lite::shared::CommonDelivery,
    /// Id of the producer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<String>,
    /// Address of the service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Id of the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_message_identifier: Option<String>, // Note: this is mandatory for idf profil
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub stop_monitoring_delivery: Vec<StopMonitoringDelivery>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub estimated_timetable_delivery: Vec<EstimatedTimetableDelivery>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub general_message_delivery: Vec<GeneralMessageDelivery>,
}
