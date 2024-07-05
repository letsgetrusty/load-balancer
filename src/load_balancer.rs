use std::{str::FromStr, sync::Arc};

use hyper::{Body, Client, Request, Uri};
use tokio::sync::RwLock;

use crate::strategies::LBStrategy;

pub struct LoadBalancer {
    client: Client<hyper::client::HttpConnector>,
    strategy: Arc<RwLock<dyn LBStrategy + Send + Sync>>,
}

impl LoadBalancer {
    pub fn new(strategy: Arc<RwLock<dyn LBStrategy + Send + Sync>>) -> Self {
        LoadBalancer {
            client: Client::new(),
            strategy,
        }
    }

    pub async fn forward_request(
        &self,
        req: Request<Body>,
    ) -> Result<hyper::Response<Body>, hyper::Error> {
        let mut worker_uri = { self.strategy.write().await.get_next_worker().to_owned() };

        let current_worker = worker_uri.clone();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        {
            self.strategy
                .write()
                .await
                .on_request_start(&current_worker)
        }

        let response = self.client.request(new_req).await;

        {
            self.strategy
                .write()
                .await
                .on_request_complete(&current_worker)
        }

        response
    }
}