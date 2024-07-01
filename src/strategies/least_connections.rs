use std::collections::HashMap;

use super::strategy::LBStrategy;

pub struct LeastConnections {
    worker_hosts: Vec<String>,
    active_connections: HashMap<String, usize>,
}

impl LeastConnections {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        LeastConnections {
            worker_hosts,
            active_connections: HashMap::new(),
        }
    }

    pub fn get_connections_count(&self, worker: &str) -> usize {
        // Get the number of active connections for the worker
        // If the worker is not in the active connections map, return 0
        *self.active_connections.get(worker).unwrap_or(&0)
    }

    pub fn add_active_connection(&mut self, worker: &str) {
        // Increment the active connection count for the worker
        let count = self
            .active_connections
            .entry(worker.to_string())
            .or_insert(0);
        *count += 1;
    }

    pub fn remove_active_connection(&mut self, worker: &str) {
        // Decrement the active connection count for the worker
        if let Some(count) = self.active_connections.get_mut(worker) {
            *count -= 1;
            if *count == 0 {
                self.active_connections.remove(worker);
            }
        }
    }
}

impl LBStrategy for LeastConnections {
    fn get_next_worker(&mut self) -> &str {
        let mut min_connections = usize::MAX;
        let mut selected_worker = "";

        for worker in &self.worker_hosts {
            // Get the number of connections for the current worker
            let connections = self.get_connections_count(worker);

            // Update the selected worker if it has fewer connections
            if connections < min_connections {
                min_connections = connections;
                selected_worker = worker;
            }
        }

        selected_worker
    }
}
