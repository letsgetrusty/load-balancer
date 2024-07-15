use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use hyper::StatusCode;

use super::TrackedRequest;

#[derive(Default, Clone)]
pub struct Metrics {
    pub total_requests: u64,
    pub total_duration: Duration,
    pub status_codes: HashMap<StatusCode, u64>,
    pub errors: u64,
    pub recent_requests: VecDeque<TrackedRequest>,
}
