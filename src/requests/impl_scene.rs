//! Implements the [Requests](`ensnano_scene::Requests`) trait for [Requests](`super::Requests`).

use crate::app_state::design_interactor::controller::clipboard::PastePosition;
use crate::controller::normal_state::Action;
use ensnano_design::{Nucl, grid::GridPosition, group_attributes::GroupPivot};
use ensnano_interactor::{
    DesignOperation,
    application::AppId,
    operation::Operation,
    selection::{CenterOfSelection, Selection},
};
use ensnano_scene::Requests as SceneRequests;
use crate::requests::Requests;
use std::sync::Arc;
use ultraviolet::{Rotor3, Vec3};

impl SceneRequests for Requests {
    fn update_operation(&mut self, op: Arc<dyn Operation>) {
        self.operation_update = Some(op);
    }

    fn set_candidate(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    fn set_selection(
        &mut self,
        selection: Vec<Selection>,
        center_of_selection: Option<CenterOfSelection>,
    ) {
        self.new_selection = Some(selection);
        self.new_center_of_selection = Some(center_of_selection);
    }

    fn set_paste_candidate(&mut self, nucl: Option<Nucl>) {
        self.new_paste_candidate = Some(nucl);
    }

    fn attempt_paste(&mut self, nucl: Option<Nucl>) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(nucl.map(PastePosition::Nucl)));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    fn paste_candidate_on_grid(&mut self, position: GridPosition) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(Some(PastePosition::GridPosition(
                position,
            ))));
    }

    fn attempt_paste_on_grid(&mut self, position: GridPosition) {
        self.keep_proceed
            .push_back(Action::PasteCandidate(Some(PastePosition::GridPosition(
                position,
            ))));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    fn xover_request(&mut self, source: Nucl, target: Nucl, _design_id: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::GeneralXover {
                source,
                target,
            }));
    }

    fn suspend_op(&mut self) {
        self.suspend_op = Some(());
    }

    fn request_center_selection(&mut self, selection: Selection, app_id: AppId) {
        self.center_selection = Some((selection, app_id));
    }

    fn undo(&mut self) {
        self.undo = Some(());
    }

    fn redo(&mut self) {
        self.redo = Some(());
    }

    fn update_builder_position(&mut self, position: isize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::MoveBuilders(
                position,
            )));
    }

    fn toggle_widget_basis(&mut self) {
        self.toggle_widget_basis = Some(());
    }

    fn apply_design_operation(&mut self, op: DesignOperation) {
        self.keep_proceed.push_back(Action::DesignOperation(op));
    }

    fn set_current_group_pivot(&mut self, pivot: GroupPivot) {
        self.keep_proceed.push_back(Action::SetGroupPivot(pivot));
    }

    fn translate_group_pivot(&mut self, translation: Vec3) {
        if let Some(Action::TranslateGroupPivot(t)) = self.keep_proceed.iter_mut().last() {
            *t = translation;
        } else {
            self.keep_proceed
                .push_back(Action::TranslateGroupPivot(translation));
        }
    }

    fn rotate_group_pivot(&mut self, rotation: Rotor3) {
        if let Some(Action::RotateGroupPivot(r)) = self.keep_proceed.iter_mut().last() {
            *r = rotation;
        } else {
            self.keep_proceed
                .push_back(Action::RotateGroupPivot(rotation));
        }
    }

    fn set_revolution_axis_position(&mut self, position: f32) {
        self.new_bezier_revolution_axis_position = Some(position as f64);
    }
}
