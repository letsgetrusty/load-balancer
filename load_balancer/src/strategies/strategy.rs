pub trait LBStrategy {
    fn get_next_worker(&mut self) -> &str;
    fn on_request_start(&mut self, worker: &str);
    fn on_request_complete(&mut self, worker: &str);
    fn get_worker_hosts(&self) -> Vec<String>;
}
