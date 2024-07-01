use std::{collections::HashMap, convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};

use hyper::{
    client::ResponseFuture,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server, Uri,
};
use tokio::sync::RwLock;
struct LoadBalancer {
    client: Client<hyper::client::HttpConnector>,
    worker_hosts: Vec<String>,
    current_worker: usize,
    active_connections: HashMap<String, usize>,
}

impl LoadBalancer {
    pub fn new(worker_hosts: Vec<String>) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }

        let active_connections = HashMap::new();

        Ok(LoadBalancer {
            client: Client::new(),
            worker_hosts,
            current_worker: 0,
            active_connections,
        })
    }

    pub async fn forward_request(&mut self, req: Request<Body>) -> (ResponseFuture, String) {
        let mut worker_uri = self.get_worker_least_connections().to_owned();

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

        // Increment the active connection count for the worker

        println!("{:?}", self.active_connections);
        self.add_active_connection(&current_worker);
        println!("{:?}", self.active_connections);
        let response = self.client.request(new_req);

        (response, current_worker)
    }

    fn get_worker(&mut self) -> &str {
        // Use a round-robin strategy to select a worker
        let worker = self.worker_hosts.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % self.worker_hosts.len();
        worker
    }

    fn get_worker_least_connections(&self) -> &str {
        let mut min_connections = usize::MAX;
        let mut selected_worker = "";

        for worker in &self.worker_hosts {
            // Get the number of connections for the current worker
            let connections = self.get_connections_count(worker);

            // Update the selected worker if it has fewer connections
            if connections < min_connections {
                min_connections = connections;
                selected_worker = worker;
            }
        }

        selected_worker
    }

    fn get_connections_count(&self, worker: &str) -> usize {
        // Get the number of active connections for the worker
        // If the worker is not in the active connections map, return 0
        *self.active_connections.get(worker).unwrap_or(&0)
    }

    fn add_active_connection(&mut self, worker: &str) {
        // Increment the active connection count for the worker
        let count = self
            .active_connections
            .entry(worker.to_string())
            .or_insert(0);
        *count += 1;
    }

    fn remove_active_connection(&mut self, worker: &str) {
        // Decrement the active connection count for the worker
        if let Some(count) = self.active_connections.get_mut(worker) {
            *count -= 1;
            if *count == 0 {
                self.active_connections.remove(worker);
            }
        }
    }
}

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, hyper::Error> {
    let (request_future, worker) = {
        let mut load_balancer = load_balancer.write().await;
        load_balancer.forward_request(req).await
        // Lock is released at the end of this scope.
        // Don't hold the lock while waiting for the response!
    };
    let result = request_future.await;

    {
        let mut load_balancer = load_balancer.write().await;
        load_balancer.remove_active_connection(&worker);
    }

    result
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:50336".to_string(),
        "http://localhost:50342".to_string(),
    ];

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    if let Err(e) = server.await {
        println!("error: {}", e);
    }
}
