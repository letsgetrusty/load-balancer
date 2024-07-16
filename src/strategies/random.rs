use super::strategy::LBStrategy;

pub struct RandomStrategy {
    worker_hosts: Vec<String>,
}

impl RandomStrategy {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        Self { worker_hosts }
    }
}

impl LBStrategy for RandomStrategy {
    fn get_next_worker(&mut self) -> &str {
        let index = rand::random::<usize>() % self.worker_hosts.len();
        &self.worker_hosts[index]
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
