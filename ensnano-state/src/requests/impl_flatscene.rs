//! Implements the [FlatSceneRequests](`ensnano_flatscene::FlatSceneRequests`) trait for [Requests](`super::Requests`).

use crate::{
    app_state::{action::Action, design_interactor::controller::clipboard::PastePosition},
    requests::Requests,
};
use crate::{
    design::{operation::DesignOperation, selection::Selection},
    utils::{application::AppId, operation::Operation},
};
use ensnano_design::nucl::Nucl;
use std::sync::Arc;
use ultraviolet::Isometry2;

impl Requests {
    pub fn xover_request(&mut self, source: Nucl, target: Nucl, _design_id: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::GeneralXover {
                source,
                target,
            }));
    }

    pub fn request_center_selection(&mut self, selection: Selection, app_id: AppId) {
        self.center_selection = Some((selection, app_id));
    }

    pub fn new_selection(&mut self, selection: Vec<Selection>) {
        self.new_selection = Some(selection);
    }

    pub fn new_candidates(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    pub fn attempt_paste(&mut self, nucl: Option<Nucl>) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(nucl.map(PastePosition::Nucl)));
        self.keep_proceed.push_back(Action::ApplyPaste);
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

    pub fn suspend_op(&mut self) {
        self.keep_proceed.push_back(Action::SuspendOp);
    }

    pub fn apply_design_operation(&mut self, op: DesignOperation) {
        self.keep_proceed.push_back(Action::DesignOperation(op));
    }

    pub fn set_paste_candidate(&mut self, candidate: Option<Nucl>) {
        self.new_paste_candidate = Some(candidate);
    }
}
