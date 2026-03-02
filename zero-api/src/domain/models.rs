use zero_core::runtime::{FailureClass, VerifyOutcome};
use zero_core::task::model::TaskStatus;

use super::error::DomainError;

pub fn parse_task_status(raw: &str) -> Result<TaskStatus, DomainError> {
    match raw {
        "pending" => Ok(TaskStatus::Pending),
        "running" => Ok(TaskStatus::Running),
        "completed" => Ok(TaskStatus::Completed),
        "failed" => Ok(TaskStatus::Failed),
        _ => Err(DomainError::BadRequest("invalid status filter".to_string())),
    }
}

pub fn parse_failure_class(raw: &str) -> Result<FailureClass, DomainError> {
    match raw {
        "provider_timeout" => Ok(FailureClass::ProviderTimeout),
        "tool_invalid_args" => Ok(FailureClass::ToolInvalidArgs),
        "planning_mismatch" => Ok(FailureClass::PlanningMismatch),
        _ => Err(DomainError::BadRequest("unknown failure_class".to_string())),
    }
}

pub fn parse_verify_outcome(raw: &str) -> Result<VerifyOutcome, DomainError> {
    match raw {
        "passed" => Ok(VerifyOutcome::Passed),
        "needs_repair" => Ok(VerifyOutcome::NeedsRepair(vec![])),
        "hard_fail" => Ok(VerifyOutcome::HardFail),
        _ => Err(DomainError::BadRequest("unknown outcome".to_string())),
    }
}
