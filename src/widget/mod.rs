pub mod top_bar;

use glow::HasContext;
use smithay_client_toolkit::{seat::pointer::PointerEvent, shell::wlr_layer::LayerSurface};

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use crate::egl::EglData;

pub trait Widget {
    fn layer_surface(&self) -> &LayerSurface;
    fn size(&self) -> (u32, u32);
    fn set_size(&mut self, width: u32, height: u32);

    /// Handles a pointer event and returns whether a redraw is needed.
    fn handle_pointer_event(&mut self, pointer_event: &PointerEvent) -> bool;

    fn egl_surface(&self) -> Option<egl::Surface>;
    fn set_egl_surface(&mut self, egl_surface: egl::Surface);
    fn wl_egl_surface(&self) -> Option<&WlEglSurface>;
    fn set_wl_egl_surface(&mut self, wl_egl_surface: WlEglSurface);

    fn draw(&mut self, egl_data: &EglData) {
        let (width, height) = self.size();
        if width == 0 || height == 0 {
            return;
        }

        if let (Some(surface), Some(gl)) = (self.egl_surface(), &egl_data.gl) {
            egl_data
                .egl_instance
                .make_current(
                    egl_data.egl_display,
                    Some(surface),
                    Some(surface),
                    Some(egl_data.egl_context),
                )
                .expect("Failed to set current");

            unsafe {
                gl.viewport(0, 0, width as i32, height as i32);
                gl.clear_color(0.0, 0.7, 0.7, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }

            if self.egl_surface().is_some() {
                egl_data
                    .egl_instance
                    .swap_buffers(egl_data.egl_display, surface)
                    .expect("Failed to swap buffer");
            }
        }
    }
}
