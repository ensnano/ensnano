use ensnano_state::utils::application::Camera3D;

use crate::app_state::AppState;
use std::borrow::Cow;

/// Represents an undoable operation.
pub(crate) struct AppStateTransition {
    /// The state that the operation was performed on.
    pub state: AppState,
    /// A label describing the operation that was performed. It is meant to be displayed in app.
    pub label: TransitionLabel,
    /// The position of the 3d scene's camera at the moment the operation was performed
    pub camera_3d: Camera3D,
}

/// A label describing an operation.
/// To create a `TransitionLabel`, use its `From<String>` or `From<'static str>` implementation
#[derive(Clone, Debug)]
pub(crate) struct TransitionLabel(Cow<'static, str>);

impl<T: Into<Cow<'static, str>>> From<T> for TransitionLabel {
    fn from(x: T) -> Self {
        Self(x.into())
    }
}

impl AsRef<str> for TransitionLabel {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub(crate) enum OkOperation {
    NotUndoable,
    Undoable {
        state: AppState,
        label: TransitionLabel,
    },
}
