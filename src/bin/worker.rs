use std::{convert::Infallible, env, net::SocketAddr, time::Duration};

use hyper::{
    body::{Bytes, Incoming},
    service::service_fn,
    Request, Response, StatusCode,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use http_body_util::Full;
use tokio::{net::TcpListener, task};

async fn worker_handler(req: Request<Incoming>, port: u16) -> Result<Response<Full<Bytes>>, Infallible> {
    let message = format!(
        "worker on port {} received {} {}",
        port,
        req.method(),
        req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(message)))
        .expect("response builder"))
}

#[tokio::main]
async fn main() {
    let port = env::args()
        .nth(1)
        .and_then(|port| port.parse().ok())
        .or_else(|| env::var("PORT").ok().and_then(|port| port.parse().ok()))
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("worker listening on http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind worker port");

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| worker_handler(req, port));
            let builder = Builder::new(TokioExecutor::new());

            if let Err(err) = builder.serve_connection(io, service).await {
                eprintln!("worker connection error: {err}");
            }
        });
    }
}
