use std::{net::SocketAddr, sync::Arc};

use load_balancer::{run_server, LoadBalancer, RoundRobin};
use tokio::{sync::RwLock, time::sleep};
use wiremock::MockServer;

pub struct TestApp {
    pub workers: Vec<MockServer>,
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let workers = vec![
            MockServer::start().await,
            MockServer::start().await,
            MockServer::start().await,
            MockServer::start().await,
        ];

        let worker_hosts = workers.iter().map(|w| w.uri()).collect::<Vec<_>>();

        let strategy = RoundRobin::new(worker_hosts);

        let load_balancer = Arc::new(LoadBalancer::new(Arc::new(RwLock::new(strategy))));

        let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(run_server(addr, load_balancer));

        // HACK: make sure the server is running before we start testing
        sleep(std::time::Duration::from_secs(1)).await;

        let address = format!("http://{}", addr);

        let http_client = reqwest::Client::new();

        Self {
            workers,
            address,
            http_client,
        }
    }

    pub async fn post_work(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/work", self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
