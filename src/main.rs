use load_balancer::{run_server, LeastConnections, LoadBalancer};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:62258".to_string(),
        "http://localhost:62260".to_string(),
    ];

    let strategy = LeastConnections::new(worker_hosts.clone());

    let load_balancer = Arc::new(LoadBalancer::new(Arc::new(RwLock::new(strategy))));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    if let Err(e) = run_server(addr, load_balancer).await {
        println!("error: {}", e);
    }
}
