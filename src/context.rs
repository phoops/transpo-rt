use gtfs_structures;

use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct GtfsRT {
    pub datetime: DateTime<Utc>,
    pub data: Vec<u8>,
}

pub struct Context {
    pub gtfs_rt: Arc<Mutex<Option<GtfsRT>>>,
    pub gtfs: gtfs_structures::Gtfs,
    pub gtfs_rt_provider_url: String,
    pub lines_of_stops: HashMap<String, HashSet<String>>,
}

fn trip_has_stop(trip: &gtfs_structures::Trip, stop: &gtfs_structures::Stop) -> bool {
    trip.stop_times.iter().any(|stop_time| {
        let stop_id = &stop_time.stop.id;
        stop_id == &stop.id || stop.parent_station.as_ref() == Some(stop_id)
    })
}

pub fn lines_of_stop(
    gtfs: &gtfs_structures::Gtfs,
    stop: &gtfs_structures::Stop,
) -> HashSet<String> {
    gtfs.trips
        .values()
        .filter(|trip| trip_has_stop(trip, stop))
        .map(|trip| trip.route_id.to_owned())
        .collect()
}