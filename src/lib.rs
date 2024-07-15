use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::RwLock;

mod decision_engine;
mod load_balancer;
mod metrics;
mod strategies;

pub use decision_engine::*;
pub use load_balancer::*;
pub use metrics::*;
pub use strategies::*;

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, hyper::Error> {
    if req.uri().path() == "/strategy" && req.method() == hyper::Method::POST {
        let body = hyper::body::to_bytes(req.into_body()).await?;
        let strategy_name = String::from_utf8_lossy(&body).to_string();
        let strategy: Arc<RwLock<dyn LBStrategy + Send + Sync>> = match strategy_name.as_str() {
            "round_robin" => Arc::new(RwLock::new(RoundRobin::new(
                load_balancer.read().await.get_worker_hosts().await,
            ))),
            "least_connections" => Arc::new(RwLock::new(LeastConnections::new(
                load_balancer.read().await.get_worker_hosts().await,
            ))),
            _ => {
                return Ok(Response::builder()
                    .status(400)
                    .body(Body::from("Invalid strategy name"))
                    .unwrap())
            }
        };
        load_balancer.write().await.set_strategy(strategy);
        Ok(Response::new(Body::from("Load balancing strategy updated")))
    } else {
        load_balancer.read().await.forward_request(req).await
    }
}

pub async fn run_server(
    addr: std::net::SocketAddr,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<(), hyper::Error> {
    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    println!("Listening on http://{}", addr);

    server.await
}
