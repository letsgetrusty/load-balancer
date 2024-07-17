use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::metrics::TrackedRequest;

use super::Metrics;

const MAX_STORED_REQUESTS: usize = 1000; // Store data for the last 1000 requests

pub struct MetricsClient {
    inner: Client<HttpConnector>,
    metrics: Arc<Mutex<Metrics>>,
}

impl MetricsClient {
    pub fn new() -> Self {
        let client = Client::new();
        MetricsClient {
            inner: client,
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
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
        metrics.total_requests += 1;
        metrics.total_duration += duration;

        if let Ok(response) = &result {
            let status = response.status();
            *metrics.status_codes.entry(status).or_insert(0) += 1;
            if !status.is_success() {
                metrics.errors += 1;
            }
        } else {
            metrics.errors += 1;
        }

        // Add to recent requests
        metrics.recent_requests.push_back(request_metric);
        if metrics.recent_requests.len() > MAX_STORED_REQUESTS {
            metrics.recent_requests.pop_front();
        }

        // Log individual request info
        println!("{} {} took: {:?}", method, uri, duration);

        result
    }

    pub async fn get_metrics(&self) -> Metrics {
        (*self.metrics.lock().await).clone()
    }

    pub async fn print_metrics(&self) {
        let metrics = self.get_metrics().await;
        println!("Total requests: {}", metrics.total_requests);
        println!(
            "Average duration: {:?}",
            metrics
                .total_duration
                .div_f64(metrics.total_requests as f64)
        );
        println!("Status code distribution: {:?}", metrics.status_codes);
        println!(
            "Error rate: {:.2}%",
            (metrics.errors as f64 / metrics.total_requests as f64) * 100.0
        );
        println!("Recent requests: {}", metrics.recent_requests.len());
    }

    pub async fn get_recent_requests(&self) -> Vec<TrackedRequest> {
        self.metrics
            .lock()
            .await
            .recent_requests
            .iter()
            .cloned()
            .collect()
    }
}
