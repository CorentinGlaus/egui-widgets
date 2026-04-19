use glow::HasContext;
use smithay_client_toolkit::{
    seat::{
        keyboard::{Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind},
    },
    shell::wlr_layer::LayerSurface,
};

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use crate::{egl::EglData, widget::Widget};

pub struct TopBar {
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub egl_surface: Option<egl::Surface>,
    pub wl_egl_surface: Option<WlEglSurface>,
    egui_ctx: egui::Context,
    raw_input: egui::RawInput,
    pointer_pos: egui::Pos2,
    modifiers: egui::Modifiers,
}

impl TopBar {
    pub fn new(layer_surface: LayerSurface) -> Self {
        Self {
            layer_surface,
            width: 0,
            height: 0,
            egl_surface: None,
            wl_egl_surface: None,
            egui_ctx: egui::Context::default(),
            raw_input: egui::RawInput::default(),
            pointer_pos: egui::Pos2::ZERO,
            modifiers: egui::Modifiers::default(),
        }
    }
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

    fn handle_pointer_event(&mut self, pointer_event: &PointerEvent) {
        match pointer_event.kind {
            PointerEventKind::Enter { .. } => {
                self.pointer_pos = egui::pos2(
                    pointer_event.position.0 as f32,
                    pointer_event.position.1 as f32,
                );
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
            }
            PointerEventKind::Leave { .. } => {
                self.raw_input.events.push(egui::Event::PointerGone);
            }
            PointerEventKind::Motion { .. } => {
                self.pointer_pos = egui::pos2(
                    pointer_event.position.0 as f32,
                    pointer_event.position.1 as f32,
                );
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
            }
            PointerEventKind::Press { button, .. } => {
                if let Some(btn) = wayland_to_egui_button(button) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
            }
            PointerEventKind::Release { button, .. } => {
                if let Some(btn) = wayland_to_egui_button(button) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
            }
            PointerEventKind::Axis {
                horizontal,
                vertical,
                ..
            } => {
                self.raw_input.events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(-horizontal.discrete as f32, -vertical.discrete as f32),
                    modifiers: self.modifiers,
                    phase: egui::TouchPhase::Move,
                });
            }
        }
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

    fn handle_keyboard_event(
        &mut self,
        keyboard_event: &smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
        if let Some(ref text) = keyboard_event.utf8 {
            self.raw_input.events.push(egui::Event::Text(text.clone()));
        }

        if let Some(key) = keysym_to_egui_key(keyboard_event.keysym) {
            self.raw_input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: self.modifiers,
            });
        }
    }

    fn handle_keyboard_modifiers(&mut self, modifiers: &Modifiers) {
        let egui_mods = egui::Modifiers {
            alt: modifiers.alt,
            ctrl: modifiers.ctrl,
            shift: modifiers.shift,
            mac_cmd: false,
            command: modifiers.ctrl,
        };

        self.modifiers = egui_mods;
        self.raw_input.modifiers = egui_mods;
    }

    fn draw_widget(&mut self, egl_data: &EglData, painter: &mut egui_glow::Painter) {
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

            self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(self.width as f32, self.height as f32),
            ));

            let full_output = self.egui_ctx.run_ui(self.raw_input.take(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if ui.button("Click me").clicked() {
                        println!("Clicked!");
                    }
                });
            });

            let clipped_primitives = self
                .egui_ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);

            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }

            painter.paint_and_update_textures(
                [self.width, self.height],
                full_output.pixels_per_point,
                &clipped_primitives,
                &full_output.textures_delta,
            );

            egl_data
                .egl_instance
                .swap_buffers(egl_data.egl_display, surface)
                .unwrap();
        }
    }
}

fn wayland_to_egui_button(button: u32) -> Option<egui::PointerButton> {
    match button {
        0x110 => Some(egui::PointerButton::Primary),
        0x111 => Some(egui::PointerButton::Secondary),
        0x112 => Some(egui::PointerButton::Middle),
        _ => None,
    }
}

fn keysym_to_egui_key(keysym: Keysym) -> Option<egui::Key> {
    use smithay_client_toolkit::seat::keyboard::Keysym as K;
    match keysym {
        K::Return | K::KP_Enter => Some(egui::Key::Enter),
        K::Tab => Some(egui::Key::Tab),
        K::BackSpace => Some(egui::Key::Backspace),
        K::Escape => Some(egui::Key::Escape),
        K::space => Some(egui::Key::Space),
        K::Left => Some(egui::Key::ArrowLeft),
        K::Right => Some(egui::Key::ArrowRight),
        K::Up => Some(egui::Key::ArrowUp),
        K::Down => Some(egui::Key::ArrowDown),
        K::Home => Some(egui::Key::Home),
        K::End => Some(egui::Key::End),
        K::Delete => Some(egui::Key::Delete),
        K::a => Some(egui::Key::A),
        K::c => Some(egui::Key::C),
        K::v => Some(egui::Key::V),
        K::x => Some(egui::Key::X),
        K::z => Some(egui::Key::Z),
        _ => None,
    }
}
