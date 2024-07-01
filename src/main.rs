use std::{convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};

use hyper::{
    client::ResponseFuture,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server, Uri,
};
use strategies::{LBStrategy, RoundRobin};
use tokio::sync::RwLock;

mod strategies;

struct LoadBalancer {
    client: Client<hyper::client::HttpConnector>,
    strategy: Box<dyn LBStrategy + Send + Sync>,
}

impl LoadBalancer {
    pub fn new(strategy: Box<dyn LBStrategy + Send + Sync>) -> Self {
        LoadBalancer {
            client: Client::new(),
            strategy,
        }
    }

    pub async fn forward_request(&mut self, req: Request<Body>) -> (ResponseFuture, String) {
        let mut worker_uri = self.strategy.get_next_worker().to_owned();

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

        let response = self.client.request(new_req);

        (response, current_worker)
    }
}

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, hyper::Error> {
    let (request_future, _) = {
        let mut load_balancer = load_balancer.write().await;
        load_balancer.forward_request(req).await
        // Lock is released at the end of this scope.
        // Don't hold the lock while waiting for the response!
    };
    let result = request_future.await;

    result
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:50336".to_string(),
        "http://localhost:50342".to_string(),
    ];

    let strategy = Box::new(RoundRobin::new(worker_hosts.clone()));

    let load_balancer = Arc::new(RwLock::new(LoadBalancer::new(strategy)));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    if let Err(e) = server.await {
        println!("error: {}", e);
    }
}
