use crate::{
    AppState,
    controller::Controller,
    data::design3d::SceneDesignReaderExt as _,
    element_selector::{CornerType, ElementSelector, SceneElement},
    view::grid::GridIntersection,
};
use ensnano_design::{
    Nucl,
    bezier_plane::{
        BezierPathId, BezierPlaneId, BezierPlaneIntersection, BezierVertex, BezierVertexId,
        ray_bezier_plane_intersection,
    },
    grid::{GridId, GridObject, GridPosition},
    helices::Axis,
};
use ensnano_interactor::selection::ActionMode;
use ultraviolet::{Vec2, Vec3};
use winit::{dpi::PhysicalPosition, keyboard::ModifiersState};

const REVOLUTION_AXIS_WIDTH: f32 = 1.;

/// The context in which an event took place.
pub(crate) struct EventContext<'a, S: AppState> {
    controller: &'a Controller<S>,
    app_state: &'a S,
    pixel_reader: &'a mut ElementSelector,
    pub cursor_position: PhysicalPosition<f64>,
}

impl<'a, S: AppState> EventContext<'a, S> {
    pub(crate) fn new(
        controller: &'a Controller<S>,
        app_state: &'a S,
        pixel_reader: &'a mut ElementSelector,
        cursor_position: PhysicalPosition<f64>,
    ) -> Self {
        Self {
            controller,
            app_state,
            pixel_reader,
            cursor_position,
        }
    }
    pub(crate) fn normalized_cursor_position(&self) -> PhysicalPosition<f64> {
        self.normalize_position(self.cursor_position)
    }

    pub(crate) fn normalize_position(&self, position: PhysicalPosition<f64>) -> PhysicalPosition<f64> {
        let normalized_x = position.x / self.controller.area_size.width as f64;
        let normalized_y = position.y / self.controller.area_size.height as f64;
        PhysicalPosition {
            x: normalized_x,
            y: normalized_y,
        }
    }

    pub(crate) fn get_projection_on_plane(&self, plane_origin: Vec3) -> Vec3 {
        let normalized_cursor = self.normalized_cursor_position();
        self.controller.camera_controller.get_projection(
            plane_origin,
            normalized_cursor.x,
            normalized_cursor.y,
            self.controller.stereography.as_ref(),
        )
    }

    pub(crate) fn get_element_under_cursor(&mut self) -> Option<SceneElement> {
        self.pixel_reader.set_selected_id(self.cursor_position)
    }

    pub(crate) fn shoot_ray(&self, point: PhysicalPosition<f64>) -> (Vec3, Vec3) {
        let normalized_point = self.normalize_position(point);
        self.controller
            .camera_controller
            .ray(normalized_point.x as f32, normalized_point.y as f32)
    }

    /// If element is a grid, get the grid disc corresponding to the grid position under the
    /// current cursor.
    /// Otherwise, return element
    pub(crate) fn convert_grid_to_grid_disc(&self, element: Option<SceneElement>) -> Option<SceneElement> {
        let normalized_position = self.normalized_cursor_position();
        if let Some(SceneElement::Grid(d_id, _)) = element {
            if let Some(intersection) = self
                .controller
                .view
                .borrow()
                .grid_intersection(normalized_position.x as f32, normalized_position.y as f32)
            {
                Some(SceneElement::GridCircle(d_id, intersection.grid_position()))
            } else {
                element
            }
        } else {
            element
        }
    }

    pub(crate) fn element_to_nucl(
        &self,
        element: Option<&SceneElement>,
        no_phantom: bool,
    ) -> Option<Nucl> {
        self.controller
            .data
            .borrow()
            .element_to_nucl(element, no_phantom)
            .map(|(n, _)| n)
    }

    pub(crate) fn get_nucl_position(&self, nucl: Nucl) -> Option<Vec3> {
        self.controller.data.borrow().get_nucl_position(nucl, 0)
    }

    /// If self is over a possible cross-over origin, return it.
    pub(super) fn get_xover_origin_under_cursor(&mut self) -> Option<XoverOrigin> {
        let element = self.get_element_under_cursor();
        let nucl = self.element_to_nucl(element.as_ref(), true)?;
        let position = self.get_nucl_position(nucl)?;
        Some(XoverOrigin {
            scene_element: element,
            nucl,
            position,
        })
    }

    pub(crate) fn can_start_builder(&self, element: Option<SceneElement>) -> Option<Nucl> {
        self.controller.data.borrow().can_start_builder(element)
    }

    /// Project the current cursor position on an axis
    pub(crate) fn get_projection_on_axis(&self, axis: Axis) -> Option<isize> {
        let normalized_cursor_position = self.normalized_cursor_position();
        self.controller.view.borrow().compute_projection_axis(
            axis,
            normalized_cursor_position.x,
            normalized_cursor_position.y,
            None,
            self.controller.stereography.is_some(),
        )
    }

    /// Get the new strand builder position corresponding to the cursor position.
    pub(crate) fn get_new_build_position(&mut self) -> Option<isize> {
        let builder = self.app_state.get_strand_builders().first()?;
        let element = self.get_element_under_cursor();

        // We can move the builder to a phantom nucl, so we do not exclude phantom nucls from the
        // search
        let no_phantom = false;

        let nucl_under_cursor = self.element_to_nucl(element.as_ref(), no_phantom);

        nucl_under_cursor
            .map(|n| n.position)
            .or_else(|| self.get_projection_on_axis(builder.get_axis()))
    }

    /// If source and dest are elements that represents nucleotides between which a xover can be
    /// made, return that pair of nucleotide.
    pub(crate) fn attempt_xover(
        &self,
        source: Option<&SceneElement>,
        target: Option<&SceneElement>,
    ) -> Option<(Nucl, Nucl, usize)> {
        self.controller.data.borrow().attempt_xover(source, target)
    }

    /// Return a reference to the current ModifiersState
    pub(crate) fn get_modifiers(&self) -> &ModifiersState {
        &self.controller.current_modifiers_state
    }

    pub(crate) fn is_editing_bezier_path(&self) -> bool {
        matches!(
            self.app_state.get_action_mode(),
            (ActionMode::EditBezierPath, _)
        )
    }

    pub(crate) fn get_bezier_vertex_being_edited(&self) -> Option<BezierVertexId> {
        self.app_state.get_selected_bezier_vertex()
    }

    pub(crate) fn get_plane_of_bezier_vertex(
        &self,
        path_id: BezierPathId,
        vertex_id: usize,
    ) -> Option<BezierPlaneId> {
        self.app_state
            .get_design_reader()
            .get_bezier_vertex(path_id, vertex_id)
            .map(|v| v.plane_id)
    }

    pub(crate) fn get_current_cursor_intersection_with_bezier_plane(
        &self,
        plane_id: BezierPlaneId,
    ) -> Option<BezierPlaneIntersection> {
        self.get_point_intersection_with_bezier_plane(plane_id, self.cursor_position)
    }

    pub(crate) fn get_point_intersection_with_bezier_plane(
        &self,
        plane_id: BezierPlaneId,
        point: PhysicalPosition<f64>,
    ) -> Option<BezierPlaneIntersection> {
        let ray = self.shoot_ray(point);
        self.app_state
            .get_design_reader()
            .get_bezier_planes()
            .get(&plane_id)
            .and_then(|p| p.ray_intersection(ray.0, ray.1))
    }

    pub(crate) fn is_pasting(&self) -> bool {
        self.app_state.is_pasting()
    }

    pub(crate) fn get_position_of_opposite_plane_corner(
        &self,
        plane_id: BezierPlaneId,
        corner_type: CornerType,
    ) -> Vec2 {
        self.app_state
            .get_design_reader()
            .get_corners_of_plane(plane_id)[corner_type.opposite().to_usize()]
    }

    /// If there is a bezier plane under the cursor, return it's identifier and the coordinates of
    /// the projection of the cursor on the plane
    pub(crate) fn get_plane_under_cursor(&self) -> Option<(BezierPlaneId, BezierPlaneIntersection)> {
        let ray = self.shoot_ray(self.cursor_position);
        ray_bezier_plane_intersection(
            self.app_state
                .get_design_reader()
                .get_bezier_planes()
                .iter(),
            ray.0,
            ray.1,
        )
    }

    pub(crate) fn cursor_is_on_revolution_axis(&self) -> bool {
        self.get_plane_under_cursor()
            .and_then(|(_, intersection)| {
                self.app_state
                    .get_revolution_axis_position()
                    .map(|x_axis| (intersection.x - x_axis as f32).abs() < REVOLUTION_AXIS_WIDTH)
            })
            .unwrap_or(false)
    }

    pub(crate) fn get_grid_intersection_with_cursor(&self) -> Option<GridIntersection> {
        let normalized_position = self.normalized_cursor_position();
        self.controller
            .view
            .borrow()
            .grid_intersection(normalized_position.x as f32, normalized_position.y as f32)
    }

    pub(crate) fn get_specific_grid_intersection(&self, grid_id: GridId) -> Option<GridIntersection> {
        let normalized_position = self.normalized_cursor_position();
        self.controller.view.borrow().specific_grid_intersection(
            normalized_position.x as f32,
            normalized_position.y as f32,
            grid_id,
        )
    }

    pub(crate) fn get_action_mode(&self) -> ActionMode {
        self.app_state.get_action_mode().0
    }

    pub(crate) fn get_object_at_grid_pos(&self, position: GridPosition) -> Option<GridObject> {
        self.controller.data.borrow().get_grid_object(position)
    }

    /// Return the SceneElement on which to place the camera rotation pivot
    pub(crate) fn get_pivot_element(&mut self) -> Option<SceneElement> {
        match self.pixel_reader.set_selected_id(self.cursor_position) {
            Some(SceneElement::Grid(d_id, g_id)) => {
                // for grids we take the precise grid position on which the user clicked.
                let mouse_x = self.cursor_position.x / self.controller.area_size.width as f64;
                let mouse_y = self.cursor_position.y / self.controller.area_size.height as f64;
                if let Some(intersection) = self
                    .controller
                    .view
                    .borrow()
                    .specific_grid_intersection(mouse_x as f32, mouse_y as f32, g_id)
                {
                    Some(SceneElement::GridCircle(d_id, intersection.grid_position()))
                } else {
                    Some(SceneElement::Grid(d_id, g_id))
                }
            }
            element => element,
        }
    }

    pub(crate) fn get_bezier_vertex(
        &self,
        path_id: BezierPathId,
        vertex_id: usize,
    ) -> Option<BezierVertex> {
        self.app_state
            .get_design_reader()
            .get_bezier_vertex(path_id, vertex_id)
    }
}

/// The element that was clicked on and that can be the origin of a crossover.
#[derive(Clone)]
pub(super) struct XoverOrigin {
    pub scene_element: Option<SceneElement>,
    pub position: Vec3,
    pub nucl: Nucl,
}
