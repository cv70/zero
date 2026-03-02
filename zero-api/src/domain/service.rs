use super::error::DomainError;

pub fn ensure_path_matches(path_id: &str, payload_id: &str) -> Result<(), DomainError> {
    if path_id != payload_id {
        return Err(DomainError::BadRequest(
            "path id must match task_id".to_string(),
        ));
    }
    Ok(())
}

pub fn ensure_plan_path_matches(path_id: &str, payload_id: &str) -> Result<(), DomainError> {
    if path_id != payload_id {
        return Err(DomainError::BadRequest(
            "path id must match plan.task_id".to_string(),
        ));
    }
    Ok(())
}

pub fn ensure_dispatchable(
    idx: usize,
    total_steps: usize,
    task_id: &str,
) -> Result<(), DomainError> {
    if idx >= total_steps {
        return Err(DomainError::InvalidState(format!(
            "no dispatchable steps for task: {task_id}"
        )));
    }
    Ok(())
}
