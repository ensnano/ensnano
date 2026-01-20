use std::sync::Arc;

use ensnano_design::nucl::Nucl;
use ensnano_state::{
    design::{operation::DesignOperation, selection::Selection},
    utils::{application::AppId, operation::Operation},
};
use ultraviolet::Isometry2;

pub trait FlatSceneRequests {
    fn xover_request(&mut self, source: Nucl, target: Nucl, design_id: usize);
    fn request_center_selection(&mut self, selection: Selection, app_id: AppId);
    fn new_selection(&mut self, selection: Vec<Selection>);
    fn new_candidates(&mut self, candidates: Vec<Selection>);
    fn attempt_paste(&mut self, nucl: Option<Nucl>);
    fn update_operation(&mut self, operation: Arc<dyn Operation>);
    fn set_isometry(&mut self, helix: usize, segment_idx: usize, isometry: Isometry2);
    fn set_visibility_helix(&mut self, helix: usize, visibility: bool);
    fn flip_group(&mut self, helix: usize);
    fn suspend_op(&mut self);
    fn apply_design_operation(&mut self, op: DesignOperation);
    fn set_paste_candidate(&mut self, candidate: Option<Nucl>);
}
