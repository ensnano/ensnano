use winit::dpi::PhysicalPosition;

/// An object mapping ElementType to DrawArea
pub trait Multiplexer {
    fn get_draw_area(&self, element_type: GuiComponentType) -> Option<DrawArea>;
    fn focused_element(&self) -> Option<GuiComponentType>;
    fn get_cursor_position(&self) -> PhysicalPosition<f64>;
    fn get_texture_view(&self, element_type: GuiComponentType) -> Option<&wgpu::TextureView>;
}
