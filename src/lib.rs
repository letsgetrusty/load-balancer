mod load_balancer;
mod strategies;

use std::{convert::Infallible, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
pub use load_balancer::*;
pub use strategies::*;

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<LoadBalancer>,
) -> Result<Response<Body>, hyper::Error> {
    load_balancer.forward_request(req).await
}

pub async fn run_server(
    addr: std::net::SocketAddr,
    load_balancer: Arc<LoadBalancer>,
) -> Result<(), hyper::Error> {
    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    println!("Listening on http://{}", addr);

    server.await
}
