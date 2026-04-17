use smithay_client_toolkit::{
    seat::pointer::{PointerEvent, PointerEventKind},
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, LayerSurface},
    },
};

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use crate::widget::Widget;

pub struct TopBar {
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub hovered: bool,
    pub egl_surface: Option<egl::Surface>,
    pub wl_egl_surface: Option<WlEglSurface>,
}

impl Widget for TopBar {
    fn layer_surface(&self) -> &LayerSurface {
        &self.layer_surface
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn handle_pointer_event(&mut self, pointer_event: &PointerEvent) -> bool {
        match pointer_event.kind {
            PointerEventKind::Enter { serial: _ } => {
                self.hovered = true;
                self.layer_surface
                    .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                self.layer_surface.commit();
                return true;
            }
            PointerEventKind::Leave { serial: _ } => {
                self.hovered = false;
                self.layer_surface
                    .set_keyboard_interactivity(KeyboardInteractivity::None);
                self.layer_surface.commit();
                return true;
            }
            PointerEventKind::Motion { time: _ } => {}
            PointerEventKind::Press {
                time: _,
                button: _,
                serial: _,
            } => {
                println!("Click pressed");
            }
            PointerEventKind::Release {
                time: _,
                button: _,
                serial: _,
            } => {
                println!("Click released")
            }
            PointerEventKind::Axis {
                time: _,
                horizontal: _,
                vertical: _,
                source: _,
            } => {}
        }
        false
    }

    fn egl_surface(&self) -> Option<khronos_egl::Surface> {
        self.egl_surface
    }

    fn set_egl_surface(&mut self, egl_surface: khronos_egl::Surface) {
        self.egl_surface = Some(egl_surface);
    }

    fn wl_egl_surface(&self) -> Option<&WlEglSurface> {
        self.wl_egl_surface.as_ref()
    }

    fn set_wl_egl_surface(&mut self, wl_egl_surface: WlEglSurface) {
        self.wl_egl_surface = Some(wl_egl_surface);
    }
}
