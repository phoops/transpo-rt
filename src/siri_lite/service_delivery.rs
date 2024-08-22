use crate::siri_lite::general_message::GeneralMessageDelivery;
use crate::siri_lite::DateTime;
use serde::{Serialize, Deserialize};
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
    #[serde(flatten)]
    pub Order: OrderWrapper,
    
    #[serde(flatten)]
    pub StopPointName: StopPointNameWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub VehicleAtStop: Option<VehicleAtStopWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub DestinationDisplay: Option<DestinationDisplayWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub AimedArrivalTime: Option<AimedArrivalTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub AimedDepartureTime: Option<AimedDepartureTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ExpectedArrivalTime: Option<ExpectedArrivalTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ExpectedDepartureTime: Option<ExpectedDepartureTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ArrivalStatus: Option<ArrivalStatusWrapper>,
}


#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedVehicleJourney {
    #[serde(flatten)]
    pub LineRef: LineRefWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub DirectionRef: Option<DirectionRefWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub JourneyPatternRef: Option<JourneyPatternRefWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub PublishedLineName: Option<PublishedLineNameWrapper>,

    pub FramedVehicleJourneyRef: FramedVehicleJourneyRef,
    
    #[serde(flatten)]
    pub OperatorRef: ServiceInfoGroup,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub VehicleRef: Option<VehicleRefWrapper>,
    
    pub EstimatedCalls: EstimatedCall,
}


#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedCall {
    #[serde(flatten)]
    pub StopPointRef: StopPointRefWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub VisitNumber: Option<VisitNumberWrapper>,
    
    #[serde(flatten)]
    pub Order: OrderWrapper,
    
    #[serde(flatten)]
    pub StopPointName: StopPointNameWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub AimedArrivalTime: Option<AimedArrivalTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub AimedDepartureTime: Option<AimedDepartureTimeWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ExpectedDepartureTime: Option<ExpectedDepartureTimeWrapper>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ExpectedArrivalTime: Option<ExpectedArrivalTimeWrapper>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfoGroup {
    /// Id of the operator
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub OperatorRef: Option<OperatorRefWrapper>,
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
    pub ServiceInfo: ServiceInfoGroup,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub JourneyPatternRef: Option<JourneyPatternRefWrapper>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoredStopVisit {
    #[serde(flatten)]
    pub MonitoringRef: MonitoringRefWrapper,
    
    #[serde(flatten)]
    pub RecordedAtTime: RecordedAtTimeWrapper,
    
    #[serde(flatten)]
    pub ItemIdentifier: ItemIdentifierWrapper,
    
    pub MonitoredVehicleJourney: MonitoredVehicleJourney,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedTimetableVisit {
    /// Id of the couple Stop / VehicleJourney
    //pub item_identifier: String,
    pub MonitoredVehicleJourney: EstimatedVehicleJourney,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct StopMonitoringDelivery {
    #[serde(flatten)]
    pub Version: VersionWrapper,
    
    #[serde(flatten)]
    pub ResponseTimestamp: ResponseTimeStampWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub RequestMessageRef: Option<RequestMessageRefWrapper>,
    
    #[serde(flatten)]
    pub Status: StatusWrapper,
    
    pub MonitoredStopVisit: Vec<MonitoredStopVisit>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedTimetableDelivery {
     #[serde(flatten)]
    pub ResponseTimestamp: ResponseTimeStampWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub RequestMessageRef: Option<RequestMessageRefWrapper>, // Note: this is mandatory for idf profil

    #[serde(flatten)]
    pub SubscriberRef: SubscriberRefWrapper,

    #[serde(flatten)]
    pub SubscriptionRef: SubscriptionRefWrapper,

    pub EstimatedJourneyVersionFrame: EstimatedJourneyVersionFrame,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
#[serde(rename_all = "PascalCase")]
pub struct EstimatedJourneyVersionFrame {
    #[serde(flatten)]
    pub RecordedAtTime: RecordedAtTimeWrapper,
    
    pub EstimatedVehicleJourney: Vec<EstimatedVehicleJourney>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceDelivery {
    #[serde(flatten)]
   pub ResponseTimestamp: ResponseTimeStampWrapper,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ProducerRef: Option<ProducerRefWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub Address: Option<AddressWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub ResponseMessageIdentifier: Option<ResponseMessageIdentifierWrapper>,
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub StopMonitoringDelivery: Vec<StopMonitoringDelivery>,
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub EstimatedTimetableDelivery: Vec<EstimatedTimetableDelivery>,
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub GeneralMessageDelivery: Vec<GeneralMessageDelivery>,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
#[serde(rename_all = "PascalCase")]
pub struct FramedVehicleJourneyRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub DataFrameRef: Option<DataFrameRefWrapper>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub DatedVehicleJourneyRef: Option<DatedVehicleJourneyRefWrapper>,
}

// Wrappers for all fields

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct DataFrameRefWrapper {
    pub DataFrameRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct DatedVehicleJourneyRefWrapper {
    pub DatedVehicleJourneyRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct ResponseTimeStampWrapper {
    pub ResponseTimestamp: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct RequestMessageRefWrapper {
    pub RequestMessageRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct SubscriberRefWrapper {
    pub SubscriberRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct SubscriptionRefWrapper {
    pub SubscriptionRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct OrderWrapper {
    pub Order: u16,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct StopPointNameWrapper {
    pub StopPointName: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct VehicleAtStopWrapper {
    pub VehicleAtStop: bool,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct DestinationDisplayWrapper {
    pub DestinationDisplay: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
pub struct AimedArrivalTimeWrapper {
    pub AimedArrivalTime: DateTime,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
pub struct AimedDepartureTimeWrapper {
    pub AimedDepartureTime: DateTime,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
pub struct ExpectedArrivalTimeWrapper {
    pub ExpectedArrivalTime: DateTime,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
pub struct ExpectedDepartureTimeWrapper {
    pub ExpectedDepartureTime: DateTime,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema)]
pub struct ArrivalStatusWrapper {
    pub ArrivalStatus: ArrivalStatus,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct LineRefWrapper {
    pub LineRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct DirectionRefWrapper {
    pub DirectionRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct JourneyPatternRefWrapper {
    pub JourneyPatternRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct PublishedLineNameWrapper {
    pub PublishedLineName: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct VehicleRefWrapper {
    pub VeichleRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct StopPointRefWrapper {
    pub StopPointRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct VisitNumberWrapper {
    pub VisitNumber: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct OperatorRefWrapper {
    pub OperatorRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct MonitoringRefWrapper {
    pub MonitoringRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct RecordedAtTimeWrapper {
    pub RecordedAtTime: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct ItemIdentifierWrapper {
    pub ItemIdentifier: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct VersionWrapper {
    pub Version: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct StatusWrapper {
    pub Status: bool,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct ProducerRefWrapper {
    pub ProducerRef: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct AddressWrapper {
    pub Address: String,
}

#[derive(Debug, Serialize, Deserialize, OpenapiSchema, Default)]
pub struct ResponseMessageIdentifierWrapper {
    pub ResponseMessageIdentifier: String,
}