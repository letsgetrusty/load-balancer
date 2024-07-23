use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::metrics::TrackedRequest;

#[derive(Default, Clone)]
pub struct Stats {
    pub total_requests: u64,
    pub average_response_time: Duration,
    pub average_error_rate: f64,
}

impl Stats {
    fn update(&mut self, new_request: &TrackedRequest) {
        self.total_requests += 1;

        // New average = old average * (n-1)/n + new value /n
        // Source: https://stackoverflow.com/questions/12636613/how-to-calculate-moving-average-without-keeping-the-count-and-data-total

        let old_response_time_average = self.average_response_time.as_secs_f64();

        let new_response_time_average = old_response_time_average
            * ((self.total_requests - 1) as f64 / self.total_requests as f64)
            + new_request.duration.as_secs_f64() / self.total_requests as f64;

        self.average_response_time = Duration::from_secs_f64(new_response_time_average);

        let old_error_rate_average = self.average_error_rate;

        let new_error_rate_average = old_error_rate_average
            * ((self.total_requests - 1) as f64 / self.total_requests as f64)
            + if new_request.is_error { 1.0 } else { 0.0 } / self.total_requests as f64;

        self.average_error_rate = new_error_rate_average;
    }
}

#[derive(Default)]
pub struct MetricsClient {
    inner: Client<HttpConnector>,
    metrics: Arc<Mutex<HashMap<String, Stats>>>, // TODO: Update key to store WorkerHost type instead of String
}

impl MetricsClient {
    pub fn new() -> Self {
        MetricsClient::default()
    }

    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let start = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();

        let result = self.inner.request(req).await;

        let duration = start.elapsed();

        // Create RequestMetric
        let request_metric = TrackedRequest {
            timestamp: start,
            method: method.clone(),
            uri: uri.clone(),
            duration,
            status: result.as_ref().ok().map(|r| r.status()),
            is_error: result.is_err()
                || result.as_ref().map_or(false, |r| !r.status().is_success()),
        };

        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics
            .entry(uri.host().unwrap().to_string())
            .or_insert(Stats::default())
            .update(&request_metric);

        // Log individual request info
        println!("{} {} took: {:?}", method, uri, duration);

        result
    }

    pub async fn get_metrics(&self) -> HashMap<String, Stats> {
        (*self.metrics.lock().await).clone()
    }
}
