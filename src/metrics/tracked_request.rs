use std::time::{Duration, Instant};

use hyper::{Method, StatusCode, Uri};

#[derive(Clone, Debug)]
pub struct TrackedRequest {
    pub timestamp: Instant,
    pub method: Method,
    pub uri: Uri,
    pub duration: Duration,
    pub status: Option<StatusCode>,
    pub is_error: bool,
}
