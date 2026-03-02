use crate::runtime::RuntimeMetricsCollector;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct PerfMonitor {
    collector: Arc<Mutex<RuntimeMetricsCollector>>,
}

impl PerfMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_task_started(&self) {
        if let Ok(mut c) = self.collector.lock() {
            c.record_task_started();
        }
    }

    pub fn record_task_succeeded(&self) {
        if let Ok(mut c) = self.collector.lock() {
            c.record_task_succeeded();
        }
    }

    pub fn record_tokens(&self, tokens: u64) {
        if let Ok(mut c) = self.collector.lock() {
            c.record_tokens(tokens);
        }
    }
}

pub fn log_perf_event(name: &str, value: f64) {
    println!("[perf] {} = {}", name, value);
}
