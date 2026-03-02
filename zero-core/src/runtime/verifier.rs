use crate::task::model::TaskSuccessContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAction {
    EnrichContext,
    RetryStep(String),
    RunTool(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Passed,
    NeedsRepair(Vec<RepairAction>),
    HardFail,
}

#[derive(Debug, Clone, Default)]
pub struct TaskVerifier;

impl TaskVerifier {
    pub fn verify(&self, contract: TaskSuccessContract, result: &str) -> VerifyOutcome {
        if contract.required_substrings.is_empty() {
            return VerifyOutcome::Passed;
        }

        let missing: Vec<String> = contract
            .required_substrings
            .into_iter()
            .filter(|s| !result.contains(s))
            .collect();

        if missing.is_empty() {
            VerifyOutcome::Passed
        } else {
            VerifyOutcome::NeedsRepair(
                missing
                    .into_iter()
                    .map(RepairAction::RetryStep)
                    .collect::<Vec<_>>(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contract() -> TaskSuccessContract {
        TaskSuccessContract {
            required_substrings: vec!["done".to_string(), "validated".to_string()],
        }
    }

    fn sample_bad_result() -> &'static str {
        "done"
    }

    #[test]
    fn failed_contract_requests_repair_loop() {
        let verifier = TaskVerifier;
        let out = verifier.verify(sample_contract(), sample_bad_result());
        assert_eq!(
            out,
            VerifyOutcome::NeedsRepair(vec![RepairAction::RetryStep("validated".to_string())])
        );
    }
}
