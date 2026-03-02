use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub task_id: String,
    pub steps: Vec<StepSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepSpec {
    pub task_id: String,
    pub step_id: String,
    pub op: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Runnable,
    Running,
    Waiting,
    Succeeded,
    Failed,
    Compensated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransitionReason {
    PlanAccepted,
    StepDispatched,
    StepCompleted,
    StepFailed,
    StepTimedOut,
    RetryScheduled,
    CompensationTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DispatchEvent {
    StepDispatched {
        task_id: String,
        step_id: String,
        op: String,
    },
}

impl DispatchEvent {
    pub fn step_dispatched(task_id: impl Into<String>, step_id: impl Into<String>, op: impl Into<String>) -> Self {
        Self::StepDispatched {
            task_id: task_id.into(),
            step_id: step_id.into(),
            op: op.into(),
        }
    }

    pub fn task_id(&self) -> &str {
        match self {
            Self::StepDispatched { task_id, .. } => task_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepResultEvent {
    StepCompleted {
        task_id: String,
        step_id: String,
        output: String,
    },
    StepFailed {
        task_id: String,
        step_id: String,
        error: String,
    },
    StepTimedOut {
        task_id: String,
        step_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::DispatchEvent;

    #[test]
    fn dispatch_event_roundtrip() {
        let evt = DispatchEvent::step_dispatched("task-1", "step-1", "agent.execute");
        let s = serde_json::to_string(&evt).unwrap();
        let back: DispatchEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back.task_id(), "task-1");
    }
}
