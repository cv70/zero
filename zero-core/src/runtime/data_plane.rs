use crate::error::ZeroError;
use crate::runtime::contracts::StepSpec;
use crate::runtime::idempotency::{IdempotencyStore, StepCacheEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub output: String,
    pub from_cache: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DataPlane {
    store: IdempotencyStore,
}

impl DataPlane {
    pub fn new_for_test() -> Self {
        Self::default()
    }

    pub async fn execute_step(&self, step: StepSpec) -> Result<StepExecutionResult, ZeroError> {
        if let Some(entry) = self.store.get(&step.idempotency_key).await {
            return Ok(StepExecutionResult {
                step_id: entry.step_id,
                output: entry.output,
                from_cache: true,
            });
        }

        let output = format!("executed:{}", step.op);
        self.store
            .put(
                step.idempotency_key,
                StepCacheEntry {
                    step_id: step.step_id.clone(),
                    output: output.clone(),
                },
            )
            .await;

        Ok(StepExecutionResult {
            step_id: step.step_id,
            output,
            from_cache: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_step(task_id: &str, step_id: &str, idempotency_key: &str) -> StepSpec {
        StepSpec {
            task_id: task_id.to_string(),
            step_id: step_id.to_string(),
            op: "agent.execute".to_string(),
            idempotency_key: idempotency_key.to_string(),
        }
    }

    #[tokio::test]
    async fn duplicate_step_is_not_executed_twice() {
        let dp = DataPlane::new_for_test();
        let spec = sample_step("task-1", "step-1", "idem-1");
        let a = dp.execute_step(spec.clone()).await.unwrap();
        let b = dp.execute_step(spec).await.unwrap();
        assert_eq!(a.step_id, b.step_id);
        assert!(b.from_cache);
    }
}
