//! Implements the [SceneRequests](`ensnano_scene::SceneRequests`) trait for [Requests](`super::Requests`).

use crate::{
    app_state::{action::Action, design_interactor::controller::clipboard::PastePosition},
    design::{
        operation::DesignOperation,
        selection::{CenterOfSelection, Selection},
    },
    requests::Requests,
    utils::application::AppId,
};
use ensnano_design::{grid::GridPosition, group_attributes::GroupPivot, nucl::Nucl};
use ultraviolet::{Rotor3, Vec3};

impl Requests {
    pub fn set_candidate(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    pub fn set_selection(
        &mut self,
        selection: Vec<Selection>,
        center_of_selection: Option<CenterOfSelection>,
    ) {
        self.new_selection = Some(selection);
        self.new_center_of_selection = Some(center_of_selection);
    }

    pub fn set_paste_candidate(&mut self, nucl: Option<Nucl>) {
        self.new_paste_candidate = Some(nucl);
    }

    pub fn attempt_paste(&mut self, nucl: Option<Nucl>) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(nucl.map(PastePosition::Nucl)));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    pub fn paste_candidate_on_grid(&mut self, position: GridPosition) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(Some(PastePosition::GridPosition(
                position,
            ))));
    }

    pub fn attempt_paste_on_grid(&mut self, position: GridPosition) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(Some(PastePosition::GridPosition(
                position,
            ))));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    pub fn xover_request(&mut self, source: Nucl, target: Nucl, _design_id: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::GeneralXover {
                source,
                target,
            }));
    }

    pub fn suspend_op(&mut self) {
        self.suspend_op = Some(());
    }

    pub fn request_center_selection(&mut self, selection: Selection, app_id: AppId) {
        self.center_selection = Some((selection, app_id));
    }

    pub fn update_builder_position(&mut self, position: isize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::MoveBuilders(
                position,
            )));
    }

    pub fn apply_design_operation(&mut self, op: DesignOperation) {
        self.keep_proceed.push_back(Action::DesignOperation(op));
    }

    pub fn set_current_group_pivot(&mut self, pivot: GroupPivot) {
        self.keep_proceed.push_back(Action::SetGroupPivot(pivot));
    }

    pub fn translate_group_pivot(&mut self, translation: Vec3) {
        if let Some(Action::TranslateGroupPivot(t)) = self.keep_proceed.iter_mut().last() {
            *t = translation;
        } else {
            self.keep_proceed
                .push_back(Action::TranslateGroupPivot(translation));
        }
    }

    pub fn rotate_group_pivot(&mut self, rotation: Rotor3) {
        if let Some(Action::RotateGroupPivot(r)) = self.keep_proceed.iter_mut().last() {
            *r = rotation;
        } else {
            self.keep_proceed
                .push_back(Action::RotateGroupPivot(rotation));
        }
    }

    pub fn set_revolution_axis_position(&mut self, position: f32) {
        self.new_bezier_revolution_axis_position = Some(position as f64);
    }
}
