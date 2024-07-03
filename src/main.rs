use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use load_balancer::LoadBalancer;
use strategies::{LeastConnections, RoundRobin};
use tokio::sync::RwLock;

mod load_balancer;
mod strategies;

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, hyper::Error> {
    let (request_future, worker) = {
        let mut load_balancer = load_balancer.write().await;
        let result = load_balancer.forward_request(req).await;
        load_balancer.on_request_start(&result.1);
        result
        // Lock is released at the end of this scope.
        // Don't hold the lock while waiting for the response!
    };
    let result = request_future.await;

    {
        let mut load_balancer = load_balancer.write().await;
        load_balancer.on_request_complete(&worker);
    }

    result
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:50336".to_string(),
        "http://localhost:50342".to_string(),
    ];

    let strategy = Box::new(LeastConnections::new(worker_hosts.clone()));

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
