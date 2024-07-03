use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use load_balancer::{LeastConnections, LoadBalancer};
use tokio::sync::RwLock;

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<LoadBalancer>,
) -> Result<Response<Body>, hyper::Error> {
    load_balancer.forward_request(req).await
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:50336".to_string(),
        "http://localhost:50342".to_string(),
    ];

    let strategy = LeastConnections::new(worker_hosts.clone());

    let load_balancer = Arc::new(LoadBalancer::new(Arc::new(RwLock::new(strategy))));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    if let Err(e) = server.await {
        println!("error: {}", e);
    }
}
