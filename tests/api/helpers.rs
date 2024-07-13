use std::sync::Arc;

use load_balancer::{run_server, LBStrategy, LeastConnections, LoadBalancer, RoundRobin};
use tokio::{sync::RwLock, time::sleep};
use wiremock::MockServer;

pub struct TestApp {
    pub workers: Vec<MockServer>,
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(number_of_workers: i8) -> TestAppBuilder {
        let mut workers = vec![];

        for _ in 0..number_of_workers {
            workers.push(MockServer::start().await);
        }

        TestAppBuilder::new(workers)
    }

    pub async fn post_work(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/work", self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_strategy(&self, strategy: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/strategy", self.address))
            .body(strategy.to_string())
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub struct TestAppBuilder {
    workers: Vec<MockServer>,
    strategy: Option<Arc<RwLock<dyn LBStrategy + Send + Sync>>>,
}

impl TestAppBuilder {
    fn new(workers: Vec<MockServer>) -> Self {
        Self {
            workers,
            strategy: None,
        }
    }

    pub fn set_round_robin_strategy(mut self) -> Self {
        let worker_hosts = self.workers.iter().map(|w| w.uri()).collect::<Vec<_>>();
        let strategy = RoundRobin::new(worker_hosts);
        self.strategy = Some(Arc::new(RwLock::new(strategy)));
        self
    }

    pub fn set_least_connections_strategy(mut self) -> Self {
        let worker_hosts = self.workers.iter().map(|w| w.uri()).collect::<Vec<_>>();
        let strategy = LeastConnections::new(worker_hosts);
        self.strategy = Some(Arc::new(RwLock::new(strategy)));
        self
    }

    pub async fn build(self) -> TestApp {
        let load_balancer = Arc::new(RwLock::new(LoadBalancer::new(self.strategy.unwrap())));

        let addr = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to port 0")
            .local_addr()
            .unwrap();

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(run_server(addr, load_balancer));

        // HACK: make sure the server is running before we start testing
        sleep(std::time::Duration::from_millis(200)).await;

        let address = format!("http://{}", addr);

        let http_client = reqwest::Client::new();

        TestApp {
            workers: self.workers,
            address,
            http_client,
        }
    }
}
