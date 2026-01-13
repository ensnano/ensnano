//! Implements the [Requests](`ensnano_flatscene::Requests`) trait for [Requests](`super::Requests`).

use crate::app_state::design_interactor::controller::clipboard::PastePosition;
use crate::controller::normal_state::Action;
use crate::requests::Requests;
use ensnano_design::{nucl::Nucl, operation::DesignOperation, selection::Selection};
use ensnano_flatscene::FlatSceneRequests;
use ensnano_utils::{application::AppId, operation::Operation};
use std::sync::Arc;
use ultraviolet::Isometry2;

impl FlatSceneRequests for Requests {
    fn xover_request(&mut self, source: Nucl, target: Nucl, _design_id: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::GeneralXover {
                source,
                target,
            }));
    }

    fn request_center_selection(&mut self, selection: Selection, app_id: AppId) {
        self.center_selection = Some((selection, app_id));
    }

    fn new_selection(&mut self, selection: Vec<Selection>) {
        self.new_selection = Some(selection);
    }

    fn new_candidates(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    fn attempt_paste(&mut self, nucl: Option<Nucl>) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(nucl.map(PastePosition::Nucl)));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    fn update_operation(&mut self, operation: Arc<dyn Operation>) {
        self.operation_update = Some(operation);
    }

    fn set_isometry(&mut self, helix: usize, segment_idx: usize, isometry: Isometry2) {
        self.keep_proceed.push_back(Action::SilentDesignOperation(
            DesignOperation::SetIsometry {
                helix,
                isometry,
                segment_idx,
            },
        ));
    }

    fn set_visibility_helix(&mut self, helix: usize, visibility: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetVisibilityHelix {
                helix,
                visible: visibility,
            },
        ));
    }

    fn flip_group(&mut self, helix: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::FlipHelixGroup {
                helix,
            }));
    }

    fn suspend_op(&mut self) {
        self.keep_proceed.push_back(Action::SuspendOp);
    }

    fn apply_design_operation(&mut self, op: DesignOperation) {
        self.keep_proceed.push_back(Action::DesignOperation(op));
    }

    fn set_paste_candidate(&mut self, candidate: Option<Nucl>) {
        self.new_paste_candidate = Some(candidate);
    }
}
