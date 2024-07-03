use super::strategy::LBStrategy;

pub struct RoundRobin {
    worker_hosts: Vec<String>,
    current_worker: usize,
}

impl RoundRobin {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        RoundRobin {
            worker_hosts,
            current_worker: 0,
        }
    }
}

impl LBStrategy for RoundRobin {
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
}
