use crate::ensnano_design::group_attributes::GroupPivot;
use crate::ensnano_interactor::{
    graphics::{DrawArea, FogParameters},
    selection::Selection,
};
use std::{path::Path, sync::Arc, time::Duration};
use ultraviolet::{Rotor3, Vec3};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Modifiers, WindowEvent},
    window::CursorIcon,
};

#[derive(Clone, Debug)]
pub struct Camera3D {
    pub position: Vec3,
    pub orientation: Rotor3,
    pub pivot_position: Option<Vec3>,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            pivot_position: None,
        }
    }
}

pub trait Application {
    type AppState;
    /// For notification about the data
    fn on_notify(&mut self, notification: Notification);
    /// The method must be called when the window is resized or when the drawing area is modified
    fn on_resize(&mut self, window_size: PhysicalSize<u32>, area: DrawArea);
    /// The methods is used to forwards the window events to applications
    fn on_event(
        &mut self,
        event: &WindowEvent,
        cursor_position: PhysicalPosition<f64>,
        app_state: &Self::AppState,
    ) -> Option<CursorIcon>;
    /// The method is used to forwards redraw_requests to applications
    fn on_redraw_request(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
    fn needs_redraw(&mut self, dt: Duration, app_state: Self::AppState) -> bool;
    fn get_position_for_new_grid(&self) -> Option<(Vec3, Rotor3)> {
        None
    }

    fn get_camera(&self) -> Option<Arc<(Camera3D, f32)>> {
        None
    }
    fn get_current_selection_pivot(&self) -> Option<GroupPivot> {
        None
    }

    fn is_split(&self) -> bool;
}

#[derive(Clone, Debug)]
/// A notification that must be send to the application
pub enum Notification {
    /// The application must show/hide the sequences
    ToggleText(bool),
    FitRequest,
    /// The designs have been deleted
    ClearDesigns,
    /// The 3d camera must face a given target
    CameraTarget((Vec3, Vec3)),
    TeleportCamera(Camera3D),
    CameraRotation(f32, f32, f32),
    CenterSelection(Selection, AppId),
    ShowTorsion(bool),
    ModifiersChanged(Modifiers),
    Split2d,
    Redim2dHelices(bool),
    Fog(FogParameters),
    WindowFocusLost,
    NewStereographicCamera(Arc<(Camera3D, f32)>),
    FlipSplitViews,
    HorizonAligned,
    ScreenShot2D(Option<Arc<Path>>),
    ScreenShot3D(Option<Arc<Path>>),
    SaveNucleotidesPositions(Option<Arc<Path>>),
    StlExport(Option<Arc<Path>>),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AppId {
    FlatScene,
    Scene,
}
