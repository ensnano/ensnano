use crate::{
    app_state::action::Action,
    design::{operation::DesignOperation, selection::Selection},
    requests::Requests,
    utils::operation::Operation,
};
use std::sync::Arc;
use ultraviolet::Isometry2;

impl Requests {
    pub fn new_selection(&mut self, selection: Vec<Selection>) {
        self.new_selection = Some(selection);
    }

    pub fn new_candidates(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    pub fn update_operation(&mut self, operation: Arc<dyn Operation>) {
        self.operation_update = Some(operation);
    }

    pub fn set_isometry(&mut self, helix: usize, segment_idx: usize, isometry: Isometry2) {
        self.keep_proceed.push_back(Action::SilentDesignOperation(
            DesignOperation::SetIsometry {
                helix,
                isometry,
                segment_idx,
            },
        ));
    }

    pub fn set_visibility_helix(&mut self, helix: usize, visibility: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetVisibilityHelix {
                helix,
                visible: visibility,
            },
        ));
    }

    pub fn flip_group(&mut self, helix: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::FlipHelixGroup {
                helix,
            }));
    }
}
