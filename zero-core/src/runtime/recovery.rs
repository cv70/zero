#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    ProviderTimeout,
    ToolInvalidArgs,
    PlanningMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDecision {
    RetryFast,
    RetryWithPatchedArgs,
    DowngradeModel,
    ReplanLocal,
    Compensate,
    Abort,
}

#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    pub provider_retry_budget: u8,
    pub tool_retry_budget: u8,
    pub planning_retry_budget: u8,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            provider_retry_budget: 1,
            tool_retry_budget: 1,
            planning_retry_budget: 1,
        }
    }
}

impl RecoveryPolicy {
    pub fn on_failure(&self, class: FailureClass, attempt: u8) -> RecoveryDecision {
        match class {
            FailureClass::ProviderTimeout => {
                if attempt <= self.provider_retry_budget {
                    RecoveryDecision::RetryFast
                } else if attempt == self.provider_retry_budget + 1 {
                    RecoveryDecision::DowngradeModel
                } else {
                    RecoveryDecision::Abort
                }
            }
            FailureClass::ToolInvalidArgs => {
                if attempt <= self.tool_retry_budget {
                    RecoveryDecision::RetryWithPatchedArgs
                } else {
                    RecoveryDecision::Compensate
                }
            }
            FailureClass::PlanningMismatch => {
                if attempt <= self.planning_retry_budget {
                    RecoveryDecision::ReplanLocal
                } else {
                    RecoveryDecision::Abort
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_timeout_triggers_retry_then_downgrade() {
        let policy = RecoveryPolicy::default();
        let decision1 = policy.on_failure(FailureClass::ProviderTimeout, 1);
        let decision2 = policy.on_failure(FailureClass::ProviderTimeout, 2);
        assert_eq!(decision1, RecoveryDecision::RetryFast);
        assert_eq!(decision2, RecoveryDecision::DowngradeModel);
    }
}
