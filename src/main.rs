use std::{net::SocketAddr, str::FromStr, sync::Arc};

use hyper::{body::Incoming, service::service_fn, Request, Response, Uri};
use hyper::server::conn::http1;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error as ClientError, ResponseFuture};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::sync::RwLock;
use tokio::{net::TcpListener, task};

struct LoadBalancer {
    client: Client<HttpConnector, Incoming>,
    worker_hosts: Vec<String>,
    current_worker: usize,
}

impl LoadBalancer {
    pub fn new(worker_hosts: Vec<String>) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }

        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Ok(LoadBalancer {
            client,
            worker_hosts,
            current_worker: 0,
        })
    }

    pub fn forward_request(&mut self, req: Request<Incoming>) -> ResponseFuture {
        let mut worker_uri = self.get_worker().to_owned();

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

        self.client.request(new_req)
    }

    fn get_worker(&mut self) -> &str {
        // Use a round-robin strategy to select a worker
        let worker = self.worker_hosts.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % self.worker_hosts.len();
        worker
    }
}

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Incoming>, ClientError> {
    load_balancer.write().await.forward_request(req).await
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
    ];

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("load balancer listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");
        let load_balancer = load_balancer.clone();

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle(req, load_balancer.clone()));

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("error: {}", e);
            }
        });
    }
}
