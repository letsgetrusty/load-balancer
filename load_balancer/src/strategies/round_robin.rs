use super::strategy::LBStrategy;

pub struct RoundRobinStrategy {
    worker_hosts: Vec<String>,
    current_worker: usize,
}

impl RoundRobinStrategy {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        Self {
            worker_hosts,
            current_worker: 0,
        }
    }
}

impl LBStrategy for RoundRobinStrategy {
    fn get_next_worker(&mut self) -> &str {
        let worker = self.worker_hosts.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % self.worker_hosts.len();
        worker
    }

    fn on_request_start(&mut self, _: &str) {
        // no-op
    }

    fn on_request_complete(&mut self, _: &str) {
        // no-op
    }

    fn get_worker_hosts(&self) -> Vec<String> {
        self.worker_hosts.clone()
    }
}
