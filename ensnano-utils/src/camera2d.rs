/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
//! This modules defines a 2D camera for the FlatScene.
//!
//! The [Globals] struct contains the value that must be send to the GPU to compute the view
//! matrix. The [Camera2D] struct modifies a [Globals] attribute and perform some view <-> world
//! coordinate conversion.

use ensnano_design::{Rotor2, Vec2};
use ensnano_interactor::consts::MAX_ZOOM_2D;
use iced_winit::winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

/// A 2D camera for the FlatScene.
pub struct Camera2D {
    globals: Globals,
    was_updated: bool,
    old_globals: Globals,
    /// Indicates whether this camera represents the bottom pane.
    pub bottom: bool,
}

impl Camera2D {
    pub fn new(globals: Globals, bottom: bool) -> Self {
        Self {
            old_globals: globals,
            globals,
            was_updated: true,
            bottom,
        }
    }

    pub fn from_resolution(resolution: [f32; 2], bottom: bool) -> Self {
        Camera2D::new(Globals::from_resolution(resolution), bottom)
    }
}

/// Movement mechanism.
///
/// The movement can be decomposed in multiple steps.
///
impl Camera2D {
    /// Return true if the globals have been modified since the last time `self.get_update()` was
    /// called.
    pub fn was_updated(&self) -> bool {
        self.was_updated
    }

    fn rotation_sign(&self) -> f32 {
        self.globals.symmetry.x * self.globals.symmetry.y * -1.0
    }

    pub fn apply_symmetry_x(&mut self) {
        self.globals.symmetry.x *= -1.0;
        self.end_movement();
    }

    pub fn apply_symmetry_y(&mut self) {
        self.globals.symmetry.y *= -1.0;
        self.end_movement();
    }

    pub fn tilt_right(&mut self) {
        self.globals.tilt -= std::f32::consts::PI / 12. * self.rotation_sign();
        self.end_movement();
    }

    pub fn tilt_left(&mut self) {
        self.globals.tilt += std::f32::consts::PI / 12. * self.rotation_sign();
        self.end_movement();
    }

    /// Return the globals
    pub fn get_globals(&self) -> &Globals {
        &self.globals
    }

    /// Return the globals if self was updated,
    pub fn update(&mut self) -> Option<&Globals> {
        if self.was_updated {
            self.was_updated = false;
            Some(&self.globals)
        } else {
            None
        }
    }

    /// Moves the camera, according to a mouse movement expressed in *normalized screen
    /// coordinates*
    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) -> (f32, f32) {
        let (x, y) = self.transform_vec(delta_x, delta_y);
        self.translate_by_vec(x, y);
        (x, y)
    }

    /// Translate self by a vector expressed in world coordinates
    pub fn translate_by_vec(&mut self, x: f32, y: f32) {
        self.globals.scroll_offset[0] = self.old_globals.scroll_offset[0] - x;
        self.globals.scroll_offset[1] = self.old_globals.scroll_offset[1] - y;
        self.was_updated = true;
    }

    /// Perform a zoom so that the point under the cursor stays at the same position on display
    pub fn process_scroll(
        &mut self,
        delta: &MouseScrollDelta,
        cursor_position: PhysicalPosition<f64>,
    ) {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => *scroll,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                (*scroll as f32) / 100.
            }
        }
        .min(1.)
        .max(-1.);
        let mult_const = 1.25_f32.powf(scroll);
        let fixed_point =
            Vec2::from(self.screen_to_world(cursor_position.x as f32, cursor_position.y as f32));
        self.globals.zoom *= mult_const;
        self.globals.zoom = self.globals.zoom.min(MAX_ZOOM_2D);
        let delta = fixed_point
            - Vec2::from(self.screen_to_world(cursor_position.x as f32, cursor_position.y as f32));
        self.globals.scroll_offset[0] += delta.x;
        self.globals.scroll_offset[1] += delta.y;
        self.end_movement();
        log::info!("zoom = {}", self.globals.zoom);
        self.was_updated = true;
    }

    pub fn zoom_closer(&mut self) {
        self.globals.zoom = self.globals.zoom.max(MAX_ZOOM_2D / 2.);
    }

    /// Discrete zoom on the scene
    #[allow(dead_code)]
    pub fn zoom_in(&mut self) {
        self.globals.zoom *= 1.25;
        self.was_updated = true;
    }

    /// Discrete zoom out of the scene
    #[allow(dead_code)]
    pub fn zoom_out(&mut self) {
        self.globals.zoom *= 0.8;
        self.was_updated = true;
    }

    /// Notify the camera that the current movement is over.
    pub fn end_movement(&mut self) {
        self.old_globals = self.globals;
        self.was_updated = true;
    }

    /// Notify the camera that the size of the drawing area has been modified
    pub fn resize(&mut self, res_x: f32, res_y: f32) {
        self.globals.resolution[0] = res_x;
        self.globals.resolution[1] = res_y;
        self.was_updated = true;
    }

    pub fn set_center(&mut self, center: Vec2) {
        self.globals.scroll_offset = center.into();
        self.was_updated = true;
        self.end_movement();
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.globals.zoom = zoom;
    }

    /// Convert a *vector* in screen coordinate to a vector in world coordinate. (Does not apply
    /// the translation)
    fn transform_vec(&self, mut x: f32, mut y: f32) -> (f32, f32) {
        x *= self.globals.symmetry.x;
        y *= self.globals.symmetry.y;
        let vec = Vec2::new(
            self.globals.resolution[0] * x / self.globals.zoom,
            self.globals.resolution[1] * y / self.globals.zoom,
        )
        .rotated_by(self.rotation().reversed());
        vec.into()
    }

    pub fn rotation(&self) -> Rotor2 {
        Rotor2::from_angle(self.globals.tilt)
    }

    /// Convert a *point* in screen ([0, x_res] * [0, y_res]) coordinate to a point in world coordiantes.
    pub fn screen_to_world(&self, x_screen: f32, mut y_screen: f32) -> (f32, f32) {
        if self.bottom {
            y_screen -= self.globals.resolution[1];
        }
        let center_to_point_x = x_screen / self.globals.resolution[0] - 0.5;
        let center_to_point_y = y_screen / self.globals.resolution[1] - 0.5;
        let (x, y) = self.transform_vec(center_to_point_x, center_to_point_y);

        (
            (self.globals.scroll_offset[0] + x),
            (self.globals.scroll_offset[1] + y),
        )
            .into()
    }

    pub fn norm_screen_to_world(&self, x_normed: f32, y_normed: f32) -> (f32, f32) {
        if self.bottom {
            self.screen_to_world(
                x_normed * self.globals.resolution[0],
                (y_normed + 1.) * self.globals.resolution[1],
            )
        } else {
            self.screen_to_world(
                x_normed * self.globals.resolution[0],
                y_normed * self.globals.resolution[1],
            )
        }
    }

    /// Convert a *point* in world coordinates to a point in normalized screen ([0, 1] * [0, 1]) coordinates
    pub fn world_to_norm_screen(&self, x_world: f32, y_world: f32) -> (f32, f32) {
        // The screen coordinates have the y axis pointed down, and so does the 2d world
        // coordinates. So we do not flip the y axis.
        let temp = Vec2::new(
            x_world - self.globals.scroll_offset[0],
            y_world - self.globals.scroll_offset[1],
        )
        .rotated_by(self.rotation());
        let coord_ndc = Vec2::new(
            temp.x * 2. * self.globals.zoom / self.globals.resolution[0] * self.globals.symmetry.x,
            temp.y * 2. * self.globals.zoom / self.globals.resolution[1] * self.globals.symmetry.y,
        );
        ((coord_ndc.x + 1.) / 2., (coord_ndc.y + 1.) / 2.)
    }

    /// Set the globals parameters to ensure that the whole rectangle is visible, taking into
    /// account the "black stripes" that surround the 2D view.
    ///
    /// The camera's view will be centered on `rectangle`'s center.
    pub fn fit_center(&mut self, mut rectangle: FitRectangle) {
        rectangle
            .ensure_min_size([20., 35.], [0.25, 0.14285715])
            .adjust_height(1.1, 0.5);

        // Pick the largest zoom factor that makes it possible to see the whole width and the
        // whole height of the rectangle.
        let zoom_x = self.globals.resolution[0] / rectangle.width();
        let zoom_y = self.globals.resolution[1] / rectangle.height();
        if zoom_x < zoom_y {
            self.globals.zoom = zoom_x;
        } else {
            self.globals.zoom = zoom_y;
        }

        // Center the view of the camera on the center of the rectangle.
        let [center_x, center_y] = rectangle.center();
        self.globals.scroll_offset[0] = center_x;
        self.globals.scroll_offset[1] = center_y;

        self.was_updated = true;
        self.end_movement();
    }

    /// Set the globals parameters to ensure that the whole rectangle is visible.
    ///
    /// The camera's top left corner will match `rectangle`'s top left corner.
    pub fn fit_top_left(&mut self, mut rectangle: FitRectangle) {
        rectangle
            .ensure_min_size([20., 35.], [0.25, 0.14285715])
            .adjust_height(1.1, 0.5);
        let zoom_x = self.globals.resolution[0] / rectangle.width();
        let zoom_y = self.globals.resolution[1] / rectangle.height();
        let mut excess_height = 0.;
        if zoom_x < zoom_y {
            self.globals.zoom = zoom_x;

            let seen_height = self.globals.resolution[1] / zoom_x;
            excess_height = seen_height - rectangle.height();
        } else {
            self.globals.zoom = zoom_y;
        }
        let [center_x, center_y] = rectangle.center();
        self.globals.scroll_offset[0] = center_x;
        self.globals.scroll_offset[1] = center_y + excess_height / 2.;
        self.was_updated = true;
        self.end_movement();
    }

    pub fn can_see_world_point(&self, point: Vec2) -> bool {
        let normalized_coord = self.world_to_norm_screen(point.x, point.y);
        normalized_coord.0 >= 0.
            && normalized_coord.0 <= 1.
            && normalized_coord.1 >= 0.015
            && normalized_coord.1 <= 1. - 0.015
    }

    pub fn get_visible_rectangle(&self) -> FitRectangle {
        let top_left: Vec2 = self.screen_to_world(0., 0.).into();
        let bottom_right: Vec2 = self.norm_screen_to_world(1., 1.).into();

        FitRectangle {
            x_min: top_left.x,
            x_max: bottom_right.x,
            y_min: top_left.y,
            y_max: bottom_right.y,
        }
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.globals, &mut other.globals);
        self.was_updated = true;
        other.was_updated = true;
    }
}

/// Values that must be send to the GPU to compute the view.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub resolution: [f32; 2],
    pub scroll_offset: [f32; 2],
    pub zoom: f32,
    pub tilt: f32,
    pub symmetry: Vec2,
}

impl Globals {
    pub fn from_resolution_and_corners(
        resolution: [f32; 2],
        top_left: Vec2,
        bottom_right: Vec2,
    ) -> Self {
        Self {
            resolution,
            scroll_offset: [
                (top_left.x + bottom_right.x) / 2.,
                (top_left.y + bottom_right.y) / 2.,
            ],
            zoom: f32::min(
                resolution[0] / (top_left.x - bottom_right.x).abs(),
                resolution[1] / (top_left.y - bottom_right.y).abs(),
            ),
            tilt: 0.0,
            symmetry: [1., 1.].into(),
        }
    }

    pub fn from_resolution(resolution: [f32; 2]) -> Self {
        Self {
            resolution,
            scroll_offset: [10.0, 40.0],
            zoom: 16.0,
            tilt: 0.0,
            symmetry: [1., 1.].into(),
        }
    }

    pub fn from_corners<F>(top_left: Vec2, bottom_right: Vec2, compute_resolution: F) -> Self
    where
        F: Fn([f32; 2]) -> [f32; 2],
    {
        log::debug!("Corners: {:?}, {:?}", top_left, bottom_right);
        let size = [
            (bottom_right.x - top_left.x).abs(),
            (bottom_right.y - top_left.y).abs(),
        ];
        Self::from_resolution_and_corners(compute_resolution(size), top_left, bottom_right)
    }
}

/// A structure to compute appropriate vews of the flat scene.
#[derive(Debug, Clone, Copy)]
pub struct FitRectangle {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

/// Creation and basic fitting methods.
impl FitRectangle {
    /// Create a new [FitRectangle] that fits only `point`.
    ///
    /// Use as a starting point, to add new points.
    pub fn from_point(point: impl Into<[f32; 2]>) -> Self {
        let [x, y] = point.into();
        Self {
            x_min: x,
            x_max: x,
            y_min: y,
            y_max: y,
        }
    }
    /// Add a new point to include into the [FitRectangle].
    pub fn add_point(&mut self, point: impl Into<[f32; 2]>) -> Self {
        let [x, y] = point.into();
        self.x_min = f32::min(self.x_min, x);
        self.x_max = f32::max(self.x_max, x);
        self.y_min = f32::min(self.y_min, y);
        self.y_max = f32::max(self.y_max, y);
        *self
    }
    pub fn from_points(points: impl IntoIterator<Item = [f32; 2]>) -> Option<Self> {
        let mut points = points.into_iter();
        if let Some(point) = points.next() {
            let mut rect = Self::from_point(point);
            for point in points {
                rect.add_point(point);
            }
            Some(rect)
        } else {
            None
        }
    }
}

/// Introspection.
impl FitRectangle {
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }
    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }
    pub fn center(&self) -> [f32; 2] {
        [
            (self.x_min + self.x_max) / 2.,
            (self.y_min + self.y_max) / 2.,
        ]
    }
    pub fn top_left(&self) -> [f32; 2] {
        [self.x_min, self.y_max]
    }

    pub fn bottom_right(&self) -> [f32; 2] {
        [self.x_max, self.y_min]
    }
}

/// Special methods.
impl FitRectangle {
    /// Ensure a minimal rectangle size.
    ///
    /// If the width or height must be increased, use the given center of mass.
    fn ensure_min_size(
        mut self,
        [min_width, min_height]: [f32; 2],
        [x_center, y_center]: [f32; 2],
    ) -> Self {
        if self.width() <= min_width {
            let diff = min_width - self.width();
            self.x_min -= diff * x_center;
            self.x_max += diff * (1.0 - x_center);
        }
        if self.height() <= min_height {
            let diff = min_height - self.height();
            self.y_min -= diff * y_center;
            self.y_max += diff * (1.0 - y_center);
        }
        self
    }

    /// Multiply the height of a rectangle by `factor` while preserving it's center of mass
    fn adjust_height(mut self, factor: f32, y_center: f32) -> Self {
        self.y_min -= factor * y_center;
        self.y_max += factor * (1.0 - y_center);
        self
    }
}

/// Needed by controller.
impl FitRectangle {
    pub fn split_vertically(self) -> (Self, Self) {
        self.ensure_min_size([20., 35.], [0.25, 0.14285715]);
        let Self {
            x_min,
            x_max,
            y_min,
            y_max,
        } = self;
        let y_mid = (y_min + y_max) / 2.0;
        (
            Self {
                x_min,
                x_max,
                y_min: y_mid,
                y_max,
            },
            Self {
                x_min,
                x_max,
                y_min,
                y_max: y_mid,
            },
        )
    }

    pub fn double_height(mut self) -> Self {
        self.ensure_min_size([20., 35.], [0.25, 0.14285715]);
        self.y_max += 2.0 * self.height();
        self
    }
}

impl FitRectangle {
    /// An initial rectangle to give to the software at startup.
    pub const INITIAL_RECTANGLE: Self = Self {
        x_min: -7.,
        x_max: 50.,
        y_min: -4.,
        y_max: 8.,
    };
}
