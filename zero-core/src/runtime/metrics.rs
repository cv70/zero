#[derive(Debug, Clone, Default)]
pub struct RuntimeMetrics {
    pub tasks_per_min: f64,
    pub task_success_rate: f64,
    pub token_per_task: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeMetricsCollector {
    started_tasks: u64,
    succeeded_tasks: u64,
    total_tokens: u64,
}

impl RuntimeMetricsCollector {
    pub fn record_task_started(&mut self) {
        self.started_tasks += 1;
    }

    pub fn record_task_succeeded(&mut self) {
        self.succeeded_tasks += 1;
    }

    pub fn record_tokens(&mut self, tokens: u64) {
        self.total_tokens += tokens;
    }

    pub fn snapshot(&self) -> RuntimeMetrics {
        let started = self.started_tasks.max(1) as f64;
        RuntimeMetrics {
            tasks_per_min: self.started_tasks as f64,
            task_success_rate: self.succeeded_tasks as f64 / started,
            token_per_task: self.total_tokens as f64 / started,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
        }
    }
}
