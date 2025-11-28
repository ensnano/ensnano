use crate::{
    app_state::AppState,
    ensnano_interactor::{application::Application, graphics::GuiComponentType},
    multiplexer::Multiplexer,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::CursorIcon,
};

/// The scheduler is responsible for running the different applications
pub struct Scheduler {
    applications: HashMap<GuiComponentType, Arc<Mutex<dyn Application<AppState = AppState>>>>,
    needs_redraw: Vec<GuiComponentType>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            applications: HashMap::new(),
            needs_redraw: Vec::new(),
        }
    }

    pub fn add_application(
        &mut self,
        application: Arc<Mutex<dyn Application<AppState = AppState>>>,
        element_type: GuiComponentType,
    ) {
        self.applications.insert(element_type, application);
    }

    /// Forwards an event to the appropriate application
    pub fn forward_event(
        &mut self,
        event: &WindowEvent,
        area: GuiComponentType,
        cursor_position: PhysicalPosition<f64>,
        app_state: AppState,
    ) -> Option<CursorIcon> {
        let app = self.applications.get_mut(&area)?;
        app.lock()
            .unwrap()
            .on_event(event, cursor_position, &app_state)
    }

    pub fn check_redraw(
        &mut self,
        multiplexer: &Multiplexer,
        dt: Duration,
        app_state: AppState,
    ) -> bool {
        log::debug!("Scheduler checking redraw");
        self.needs_redraw.clear();
        for (area, app) in &mut self.applications {
            if multiplexer.is_showing(area)
                && app.lock().unwrap().needs_redraw(dt, app_state.clone())
            {
                self.needs_redraw.push(*area);
            }
        }
        !self.needs_redraw.is_empty()
    }

    /// Request an application to draw on a texture
    pub fn draw_apps(&mut self, encoder: &mut wgpu::CommandEncoder, multiplexer: &Multiplexer) {
        for area in &self.needs_redraw {
            let app = self.applications.get_mut(area).unwrap();
            if let Some(target) = multiplexer.get_texture_view(*area) {
                app.lock().unwrap().on_redraw_request(encoder, target);
            }
        }
    }

    /// Notify all applications that the size of the window has been modified
    pub fn forward_new_size(&mut self, window_size: PhysicalSize<u32>, multiplexer: &Multiplexer) {
        if window_size.height > 0 && window_size.width > 0 {
            for (area, app) in &mut self.applications {
                if let Some(draw_area) = multiplexer.get_draw_area(*area) {
                    app.lock().unwrap().on_resize(window_size, draw_area);
                    self.needs_redraw.push(*area);
                }
            }
        }
    }
}
