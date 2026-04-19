pub mod top_bar;

use glow::HasContext;
use smithay_client_toolkit::{
    seat::{
        keyboard::{KeyEvent, Modifiers},
        pointer::PointerEvent,
    },
    shell::wlr_layer::LayerSurface,
};

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use crate::egl::EglData;

pub trait Widget {
    fn layer_surface(&self) -> &LayerSurface;
    fn size(&self) -> (u32, u32);
    fn set_size(&mut self, width: u32, height: u32);

    fn handle_pointer_event(&mut self, pointer_event: &PointerEvent);
    fn handle_keyboard_event(&mut self, keyboard_event: &KeyEvent);
    fn handle_keyboard_modifiers(&mut self, modifiers: &Modifiers);

    fn egl_surface(&self) -> Option<egl::Surface>;
    fn set_egl_surface(&mut self, egl_surface: egl::Surface);
    fn wl_egl_surface(&self) -> Option<&WlEglSurface>;
    fn set_wl_egl_surface(&mut self, wl_egl_surface: WlEglSurface);

    fn draw_widget(&mut self, egl_data: &EglData, painter: &mut egui_glow::Painter);
}
