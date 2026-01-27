use crate::app_state::AppState;

pub trait AppStateOperation {
    fn apply(&mut self, state: &mut AppState);
}

impl<F: Fn(&mut AppState)> AppStateOperation for F {
    fn apply(&mut self, state: &mut AppState) {
        self(state);
    }
}
