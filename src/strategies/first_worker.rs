use super::strategy::LBStrategy;

pub struct FirstWorkerStrategy {
    worker_hosts: Vec<String>,
}

impl FirstWorkerStrategy {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        Self { worker_hosts }
    }
}

impl LBStrategy for FirstWorkerStrategy {
    fn get_next_worker(&mut self) -> &str {
        &self.worker_hosts[0]
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
