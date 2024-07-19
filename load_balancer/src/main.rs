use config::Config;
use load_balancer::{run_server, DecisionEngine, LoadBalancer, MetricsClient, RoundRobinStrategy};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct ConfigSettings {
    worker_hosts: Vec<String>,
}

#[tokio::main]
async fn main() {
    let config = Config::builder()
        // Add in `/config.json`
        .add_source(config::File::with_name("config"))
        // Add in settings from the environment (with a prefix of LB)
        // Eg.. `LB_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let config_settings: ConfigSettings = config.try_deserialize().unwrap();

    let strategy = Arc::new(RwLock::new(RoundRobinStrategy::new(
        config_settings.worker_hosts.clone(),
    )));
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
