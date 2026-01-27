//! Test suite for the `MainState` structure

use ensnano_design::nucl::Nucl;
use ensnano_state::{
    app_state::{
        AppState,
        design_interactor::controller::clipboard::{CopyOperation, PastePosition},
    },
    design::{operation::DesignOperation, selection::Selection},
    gui::messages::GuiMessages,
    state::MainState,
    utils::application::{Application, Camera3D, Notification},
};
use ensnano_utils::{
    PastingStatus,
    graphics::{DrawArea, GuiComponentType},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use ultraviolet::{Rotor3, Vec3};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::CursorIcon,
};

struct DummyScene;

impl Application for DummyScene {
    fn on_notify(&mut self, _notification: Notification) {}

    fn needs_redraw(&mut self, _dt: Duration, _app_state: &AppState) -> bool {
        false
    }

    fn on_redraw_request(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _target: &wgpu::TextureView,
    ) {
    }

    fn is_split(&self) -> bool {
        false
    }

    fn on_event(
        &mut self,
        _event: &WindowEvent,
        _position: PhysicalPosition<f64>,
        _app_state: &AppState,
    ) -> Option<CursorIcon> {
        None
    }

    fn on_resize(&mut self, _window_size: PhysicalSize<u32>, _area: DrawArea) {}

    fn get_camera(&self) -> Option<Arc<(Camera3D, f32)>> {
        Some(Arc::new((
            Camera3D {
                position: Vec3::zero(),
                orientation: Rotor3::identity(),
                pivot_position: None,
            },
            1.0,
        )))
    }
}

fn new_state() -> MainState {
    let messages = Arc::new(Mutex::new(GuiMessages::new()));
    let mut ret = MainState::new(messages);
    ret.applications
        .insert(GuiComponentType::Scene, Arc::new(Mutex::new(DummyScene {})));
    ret
}

#[test]
fn undoable_selection() {
    let mut state = new_state();
    let selection_1 = vec![Selection::Strand(0, 0), Selection::Strand(0, 1)];
    state.update_selection(selection_1.clone(), None);
    state.update_selection(vec![], None);
    state.undo();
    assert_eq!(state.app_state.get_selection(), selection_1);
}

#[test]
fn redoable_selection() {
    let mut state = new_state();
    let selection_1 = vec![Selection::Strand(0, 0), Selection::Strand(0, 1)];
    state.update_selection(selection_1.clone(), None);
    state.undo();
    assert_eq!(state.app_state.get_selection(), vec![]);
    state.redo();
    assert_eq!(state.app_state.get_selection(), selection_1);
}

#[test]
fn empty_selections_dont_pollute_undo_stack() {
    let mut state = new_state();
    let selection_1 = vec![Selection::Strand(0, 0), Selection::Strand(0, 1)];
    state.update_selection(selection_1.clone(), None);
    state.update_selection(vec![], None);
    state.update_selection(vec![], None);
    state.undo();
    assert_eq!(state.app_state.get_selection(), selection_1);
}

#[test]
fn recolor_staple_undoable() {
    let mut state = new_state();
    state.apply_design_operation(DesignOperation::RecolorStaples);
    assert!(!state.undo_stack.is_empty());
}

/// A design with one strand h1: -1 -> 7 ; h2: -1 <- 7 ; h3: 0 -> 9 that can be pasted on
/// helices 4, 5 and 6
fn pastable_design() -> AppState {
    let path = test_path("pastable.json");
    AppState::import_design(path).ok().unwrap()
}

fn test_path(design_name: &'static str) -> PathBuf {
    let mut ret = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    ret.push("tests");
    ret.push(design_name);
    ret
}

#[test]
fn duplication_via_requests_correct_status() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Strand(0, 0)], None);
    main_state.request_duplication();
    assert_eq!(
        main_state.app_state.get_pasting_status(),
        PastingStatus::Duplication
    );
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 10,
            forward: true,
        }),
    )));
    main_state.apply_paste();
    assert_eq!(
        main_state.app_state.get_pasting_status(),
        PastingStatus::None
    );
    main_state.request_duplication();
    assert_eq!(
        main_state.app_state.get_pasting_status(),
        PastingStatus::None
    );
}

#[test]
fn duplication_via_requests_strands_are_duplicated() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Strand(0, 0)], None);
    let initial_amount = main_state
        .get_app_state()
        .get_design_interactor()
        .get_all_nucl_ids()
        .len();
    assert!(initial_amount > 0);
    main_state.request_duplication();
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 10,
            forward: true,
        }),
    )));
    main_state.apply_paste();
    main_state.update();
    let amount = main_state
        .get_app_state()
        .get_design_interactor()
        .get_all_nucl_ids()
        .len();
    assert_eq!(amount, 2 * initial_amount);
    main_state.request_duplication();
    main_state.update();
    let amount = main_state
        .get_app_state()
        .get_design_interactor()
        .get_all_nucl_ids()
        .len();
    assert_eq!(amount, 3 * initial_amount);
    main_state.request_duplication();
    main_state.update();
    let amount = main_state
        .get_app_state()
        .get_design_interactor()
        .get_all_nucl_ids()
        .len();
    assert_eq!(amount, 4 * initial_amount);
}

#[test]
fn new_selection_empties_duplication_clipboard() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Strand(0, 0)], None);
    main_state.request_duplication();
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 10,
            forward: true,
        }),
    )));
    main_state.apply_paste();
    main_state.request_duplication();
    assert_eq!(
        main_state.app_state.get_pasting_status(),
        PastingStatus::None
    );
    main_state.update_selection(vec![Selection::Strand(0, 0), Selection::Strand(0, 1)], None);
    main_state.request_duplication();
    assert_eq!(
        main_state.app_state.get_pasting_status(),
        PastingStatus::Duplication
    );
    main_state.update();
}

#[test]
fn position_paste_via_requests() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Xover(0, 0)], None);
    main_state.request_copy();
    let nucl = Nucl {
        helix: 1,
        position: 3,
        forward: true,
    };
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&nucl)
            .to_opt()
            .is_none()
    );
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(None));
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 3,
            forward: true,
        }),
    )));
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&nucl)
            .to_opt()
            .is_some()
    );
}

#[test]
fn undo_redo_copy_paste_xover() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Xover(0, 0)], None);
    main_state.request_copy();
    let nucl = Nucl {
        helix: 1,
        position: 3,
        forward: true,
    };
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(None));
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 3,
            forward: true,
        }),
    )));
    main_state.apply_copy_operation(CopyOperation::Paste);
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&nucl)
            .to_opt()
            .is_some()
    );
    main_state.undo();
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&nucl)
            .to_opt()
            .is_none()
    );
    main_state.redo();
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&nucl)
            .to_opt()
            .is_some()
    );
}

#[test]
fn undo_redo_copy_paste_xover_pasting_status() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Xover(0, 0)], None);
    main_state.request_copy();
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(None));
    assert!(main_state.app_state.get_pasting_status().is_pasting());
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 3,
            forward: true,
        }),
    )));
    assert!(main_state.app_state.get_pasting_status().is_pasting());
    main_state.apply_copy_operation(CopyOperation::Paste);
    main_state.update();
    assert!(!main_state.app_state.get_pasting_status().is_pasting());
    main_state.undo();
    main_state.update();
    assert!(!main_state.app_state.get_pasting_status().is_pasting());
    main_state.redo();
    main_state.update();
    assert!(!main_state.app_state.get_pasting_status().is_pasting());
}

#[test]
fn duplicate_xover_pasting_status() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Xover(0, 0)], None);
    main_state.request_duplication();
    assert!(main_state.app_state.get_pasting_status().is_pasting());
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(Nucl {
            helix: 1,
            position: 5,
            forward: true,
        }),
    )));
    main_state.apply_paste();
    main_state.update();
    assert!(!main_state.app_state.get_pasting_status().is_pasting());
    main_state.request_duplication();
    main_state.update();
    assert!(!main_state.app_state.get_pasting_status().is_pasting());
}

#[test]
fn duplicate_xover() {
    let mut main_state = new_state();
    let app_state = pastable_design();
    main_state.clear_app_state(app_state);
    main_state.update_selection(vec![Selection::Xover(0, 0)], None);
    main_state.request_duplication();
    let n1 = Nucl {
        helix: 1,
        position: 5,
        forward: true,
    };
    let n2 = Nucl {
        helix: 1,
        position: 3,
        forward: true,
    };
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n1)
            .to_opt()
            .is_none()
    );
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n2)
            .to_opt()
            .is_none()
    );
    main_state.apply_copy_operation(CopyOperation::PositionPastingPoint(Some(
        PastePosition::Nucl(n1),
    )));
    main_state.apply_paste();
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n1)
            .to_opt()
            .is_some()
    );
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n2)
            .to_opt()
            .is_none()
    );
    main_state.request_duplication();
    main_state.update();
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n1)
            .to_opt()
            .is_some()
    );
    assert!(
        main_state
            .app_state
            .get_design_interactor()
            .is_xover_end(&n2)
            .to_opt()
            .is_some()
    );
}

#[test]
fn default_app_state_does_not_need_save() {
    let mut main_state = new_state();
    assert!(!main_state.need_save(), "Need save before update");
    main_state.update();
    assert!(!main_state.need_save(), "Need save after update");
}

#[test]
fn no_need_to_save_after_new_design() {
    let mut main_state = new_state();
    main_state.new_design();
    assert!(!main_state.need_save(), "Need save before update");
    main_state.update();
    assert!(!main_state.need_save(), "Need save after update");
}
