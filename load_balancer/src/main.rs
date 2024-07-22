use load_balancer::{run_server, DecisionEngine, LoadBalancer, MetricsClient, RoundRobinStrategy};
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct ConfigSettings {
    worker_hosts: Vec<String>,
}

#[tokio::main]
async fn main() {
    let worker_hosts: Vec<String> = {
        let workers = env::var("LB_WORKER_HOSTS").expect("LB_WORKER_HOSTS must be set");
        workers.split(',').map(String::from).collect()
    };

    let strategy = Arc::new(RwLock::new(RoundRobinStrategy::new(worker_hosts)));
    let metrics_client = Arc::new(MetricsClient::new());
    let load_balancer = Arc::new(RwLock::new(LoadBalancer::new(
        strategy,
        metrics_client.clone(),
    )));

    let decision_engine = DecisionEngine::new(
        Arc::clone(&load_balancer),
        metrics_client,
        Some(Duration::from_secs(10)),
    );
    decision_engine.start();

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    if let Err(e) = run_server(addr, load_balancer).await {
        println!("error: {}", e);
    }
}
