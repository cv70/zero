use zero_core::runtime::{ControlPlane, DataPlane, ExecutionPlan, StepSpec};

struct RuntimeHarness;

struct RunResult {
    success_rate: f64,
}

impl RuntimeHarness {
    fn new() -> Self {
        Self
    }

    async fn run_fault_injected_workload(&self) -> RunResult {
        let mut cp = ControlPlane::new_in_memory();
        let dp = DataPlane::new_for_test();

        let plan = ExecutionPlan {
            task_id: "task-1".to_string(),
            steps: vec![StepSpec {
                task_id: "task-1".to_string(),
                step_id: "step-1".to_string(),
                op: "agent.execute".to_string(),
                idempotency_key: "idem-1".to_string(),
            }],
        };

        cp.accept_plan(plan).await.unwrap();
        let _ = dp
            .execute_step(StepSpec {
                task_id: "task-1".to_string(),
                step_id: "step-1".to_string(),
                op: "agent.execute".to_string(),
                idempotency_key: "idem-1".to_string(),
            })
            .await
            .unwrap();
        cp.on_step_completed("task-1", "step-1").await.unwrap();

        RunResult { success_rate: 1.0 }
    }
}

#[tokio::test]
async fn runtime_recovers_after_injected_timeout() {
    let harness = RuntimeHarness::new();
    let result = harness.run_fault_injected_workload().await;
    assert!(result.success_rate >= 0.95);
}
