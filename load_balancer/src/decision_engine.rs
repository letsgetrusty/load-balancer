use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{FirstWorkerStrategy, LoadBalancer, MetricsClient};
use tokio::time::{sleep, Duration};

pub struct DecisionEngine {
    load_balancer: Arc<RwLock<LoadBalancer>>,
    metrics_client: Arc<MetricsClient>,
    sleep_duration: Option<Duration>,
}

impl DecisionEngine {
    pub fn new(
        load_balancer: Arc<RwLock<LoadBalancer>>,
        metrics_client: Arc<MetricsClient>,
        sleep_duration: Option<Duration>,
    ) -> Self {
        DecisionEngine {
            load_balancer,
            metrics_client,
            sleep_duration,
        }
    }

    pub fn start(&self) {
        let load_balancer = Arc::clone(&self.load_balancer);
        let metrics_client = Arc::clone(&self.metrics_client);
        let sleep_duration = self.sleep_duration;

        tokio::spawn(async move {
            loop {
                println!("Decision Engine: Checking metrics");
                if metrics_client.get_metrics().await.total_requests > 5 {
                    println!("Decision Engine: Changing strategy to FirstWorkerStrategy");
                    let mut lb = load_balancer.write().await;
                    let strategy = Arc::new(RwLock::new(FirstWorkerStrategy::new(
                        lb.get_worker_hosts().await,
                    )));
                    lb.set_strategy(strategy);
                }

                if sleep_duration.is_some() {
                    sleep(sleep_duration.unwrap()).await;
                }
            }
        });
    }
}
