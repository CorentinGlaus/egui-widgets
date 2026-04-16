use smithay_client_toolkit::{
    seat::pointer::{PointerEvent, PointerEventKind},
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, LayerSurface},
    },
    shm::slot::Buffer,
};

use crate::widget::Widget;

pub struct TopBar {
    pub layer_surface: LayerSurface,
    pub buffer: Option<Buffer>,
    pub width: u32,
    pub height: u32,
    pub hovered: bool,
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

    fn render(&self, canvas: &mut [u8]) {
        let (r, g, b) = if self.hovered {
            (255, 100, 100)
        } else {
            (0, 180, 180)
        };
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = b;
            pixel[1] = g;
            pixel[2] = r;
            pixel[3] = 255;
        }
    }

    fn store_buffer(&mut self, buffer: Buffer) {
        self.buffer = Some(buffer);
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
}
