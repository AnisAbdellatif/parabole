use crate::model::SourceInput;
use anyhow::Result;

/// Classify text pasted by the user (PLAN.md §5.4). No network access.
pub fn detect(input: &str) -> Result<SourceInput> {
    let _ = input;
    todo!()
}
