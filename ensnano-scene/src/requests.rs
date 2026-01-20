use std::sync::Arc;

use ensnano_design::{grid::GridPosition, group_attributes::GroupPivot, nucl::Nucl};
use ensnano_state::{
    design::{
        operation::DesignOperation,
        selection::{CenterOfSelection, Selection},
    },
    utils::{application::AppId, operation::Operation},
};
use ultraviolet::{Rotor3, Vec3};

pub trait SceneRequests {
    fn update_operation(&mut self, op: Arc<dyn Operation>);
    fn apply_design_operation(&mut self, op: DesignOperation);
    fn set_candidate(&mut self, candidates: Vec<Selection>);
    fn set_paste_candidate(&mut self, nucl: Option<Nucl>);
    fn set_selection(
        &mut self,
        selection: Vec<Selection>,
        center_of_selection: Option<CenterOfSelection>,
    );
    fn paste_candidate_on_grid(&mut self, position: GridPosition);
    fn attempt_paste_on_grid(&mut self, position: GridPosition);
    fn attempt_paste(&mut self, nucl: Option<Nucl>);
    fn xover_request(&mut self, source: Nucl, target: Nucl, design_id: usize);
    fn suspend_op(&mut self);
    fn request_center_selection(&mut self, selection: Selection, app_id: AppId);
    fn undo(&mut self);
    fn redo(&mut self);
    fn update_builder_position(&mut self, position: isize);
    fn toggle_widget_basis(&mut self);
    fn set_current_group_pivot(&mut self, pivot: GroupPivot);
    fn translate_group_pivot(&mut self, translation: Vec3);
    fn rotate_group_pivot(&mut self, rotation: Rotor3);
    fn set_revolution_axis_position(&mut self, position: f32);
}
