use crate::app_state::{AppState, design_interactor::controller::OperationError};
use std::borrow::Cow;

/// The result of an AppState operation.
///
/// An operation has been successfully applied on a design, resulting in a new modified design. The
/// variants of these enums indicate different ways in which the result should be handled.
/// A save of the current design is always done before we do an operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppStateOperationOutcome {
    /// Push the previous design unto the undo stack.
    Push {
        /// A description of the operation that was applied.
        label: Cow<'static, str>,
    },
    /// An operation happened, but it is not worth putting on the undo stack.
    Replace,
    /// No operation happened.
    NoOp,
}

pub type AppStateOperationResult = Result<AppStateOperationOutcome, OperationError>;

pub trait AppStateOperation {
    fn apply(self, state: &mut AppState) -> AppStateOperationResult;
}

impl<F: FnOnce(&mut AppState) -> Result<AppStateOperationOutcome, OperationError>> AppStateOperation
    for F
{
    fn apply(self, state: &mut AppState) -> AppStateOperationResult {
        self(state)
    }
}
