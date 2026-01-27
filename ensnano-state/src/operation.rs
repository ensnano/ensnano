use crate::app_state::AppState;

pub trait AppStateOperation {
    fn apply(&mut self, state: AppState) -> AppState;
}

impl<F: Fn(AppState) -> AppState> AppStateOperation for F {
    fn apply(&mut self, state: AppState) -> AppState {
        self(state)
    }
}
