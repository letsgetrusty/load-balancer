use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{FirstWorkerStrategy, LoadBalancer, MetricsClient};
use tokio::time::{sleep, Duration};

pub struct DecisionEngine {
    load_balancer: Arc<RwLock<LoadBalancer>>,
    metrics_client: Arc<MetricsClient>,
}

impl DecisionEngine {
    pub fn new(
        load_balancer: Arc<RwLock<LoadBalancer>>,
        metrics_client: Arc<MetricsClient>,
    ) -> Self {
        DecisionEngine {
            load_balancer,
            metrics_client,
        }
    }

    pub fn start(&self) {
        let load_balancer = Arc::clone(&self.load_balancer);
        let metrics_client = Arc::clone(&self.metrics_client);

        tokio::spawn(async move {
            loop {
                println!("Decision Engine: Checking metrics");
                if metrics_client.get_metrics().await.total_requests > 5 {
                    println!("Decision Engine: Changing strategy to FirstWorkerStrategy");
                    let mut lb: tokio::sync::RwLockWriteGuard<LoadBalancer> =
                        load_balancer.write().await;
                    let strategy = Arc::new(RwLock::new(FirstWorkerStrategy::new(
                        lb.get_worker_hosts().await,
                    )));
                    lb.set_strategy(strategy);
                }

                sleep(Duration::from_secs(10)).await;
            }
        });
    }
}
