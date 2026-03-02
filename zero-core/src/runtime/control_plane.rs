use std::collections::HashMap;

use crate::error::ZeroError;
use crate::runtime::contracts::{ExecutionPlan, StepSpec, TaskState};
use crate::runtime::recovery::{FailureClass, RecoveryDecision, RecoveryPolicy};
use crate::runtime::verifier::VerifyOutcome;

pub struct ControlPlane {
    plans: HashMap<String, ExecutionPlan>,
    states: HashMap<String, TaskState>,
    completed_steps: HashMap<String, usize>,
    recovery_policy: RecoveryPolicy,
}

impl ControlPlane {
    pub fn new_in_memory() -> Self {
        Self {
            plans: HashMap::new(),
            states: HashMap::new(),
            completed_steps: HashMap::new(),
            recovery_policy: RecoveryPolicy::default(),
        }
    }

    pub fn with_recovery_policy(mut self, recovery_policy: RecoveryPolicy) -> Self {
        self.recovery_policy = recovery_policy;
        self
    }

    pub async fn accept_plan(&mut self, plan: ExecutionPlan) -> Result<(), ZeroError> {
        let task_id = plan.task_id.clone();
        self.completed_steps.insert(task_id.clone(), 0);
        self.states.insert(task_id.clone(), TaskState::Running);
        self.plans.insert(task_id, plan);
        Ok(())
    }

    pub async fn on_step_completed(
        &mut self,
        task_id: &str,
        step_id: &str,
    ) -> Result<(), ZeroError> {
        let plan = self
            .plans
            .get(task_id)
            .ok_or_else(|| ZeroError::NotFound(format!("task not found: {task_id}")))?;
        let has_step = plan.steps.iter().any(|s| s.step_id == step_id);
        if !has_step {
            return Err(ZeroError::NotFound(format!(
                "step not found for task {task_id}: {step_id}"
            )));
        }

        let completed = self.completed_steps.entry(task_id.to_string()).or_insert(0);
        *completed += 1;
        if *completed >= plan.steps.len() {
            self.states
                .insert(task_id.to_string(), TaskState::Succeeded);
        } else {
            self.states.insert(task_id.to_string(), TaskState::Running);
        }
        Ok(())
    }

    pub async fn task_state(&self, task_id: &str) -> Result<TaskState, ZeroError> {
        self.states
            .get(task_id)
            .cloned()
            .ok_or_else(|| ZeroError::NotFound(format!("task state not found: {task_id}")))
    }

    pub fn decide_recovery(&self, class: FailureClass, attempt: u8) -> RecoveryDecision {
        self.recovery_policy.on_failure(class, attempt)
    }

    pub fn handle_verification_outcome(&self, outcome: VerifyOutcome) -> RecoveryDecision {
        match outcome {
            VerifyOutcome::Passed => RecoveryDecision::Abort,
            VerifyOutcome::NeedsRepair(_) => RecoveryDecision::ReplanLocal,
            VerifyOutcome::HardFail => RecoveryDecision::Compensate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan() -> ExecutionPlan {
        ExecutionPlan {
            task_id: "task-1".to_string(),
            steps: vec![
                StepSpec {
                    task_id: "task-1".to_string(),
                    step_id: "step-1".to_string(),
                    op: "agent.execute".to_string(),
                    idempotency_key: "idem-1".to_string(),
                },
                StepSpec {
                    task_id: "task-1".to_string(),
                    step_id: "step-2".to_string(),
                    op: "tool.execute".to_string(),
                    idempotency_key: "idem-2".to_string(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn state_machine_advances_on_step_completed() {
        let mut cp = ControlPlane::new_in_memory();
        cp.accept_plan(sample_plan()).await.unwrap();
        cp.on_step_completed("task-1", "step-1").await.unwrap();
        assert_eq!(cp.task_state("task-1").await.unwrap(), TaskState::Running);
    }
}
