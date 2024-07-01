pub trait LBStrategy {
    fn get_next_worker(&mut self) -> &str;
}
