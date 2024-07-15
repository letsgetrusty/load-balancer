use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{LoadBalancer, MetricsClient};
use tokio::time::{sleep, Duration};

struct DecisionEngine {
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
                // Decide if the load balancing algorithm should change
                // ...

                // Sleep for 60 seconds
                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}
