use load_balancer::{
    run_server, DecisionEngine, LeastConnectionsStrategy, LoadBalancer, MetricsClient,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:62258".to_string(),
        "http://localhost:62260".to_string(),
    ];

    let strategy = Arc::new(RwLock::new(LeastConnectionsStrategy::new(
        worker_hosts.clone(),
    )));
    let metrics_client = Arc::new(MetricsClient::new());
    let load_balancer = Arc::new(RwLock::new(LoadBalancer::new(
        strategy,
        metrics_client.clone(),
    )));

    let decision_engine = DecisionEngine::new(Arc::clone(&load_balancer), metrics_client);
    decision_engine.start();

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    if let Err(e) = run_server(addr, load_balancer).await {
        println!("error: {}", e);
    }
}
