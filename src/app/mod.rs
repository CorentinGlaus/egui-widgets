pub mod builder;

use std::sync::Arc;

use khronos_egl as egl;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, Proxy, QueueHandle,
        protocol::{wl_output, wl_surface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::KeyboardHandler,
        pointer::{PointerEvent, PointerHandler},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_egl::WlEglSurface;

use crate::{egl::EglData, widget::Widget};

pub struct App {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub egl_data: EglData,
    painter: Option<egui_glow::Painter>,

    pub widgets: Vec<Box<dyn Widget>>,
    selected_widget: Option<usize>,

    pub should_exit: bool,

    // Needed to keep alive
    pub _compositor_state: CompositorState,
    pub _layer_shell: LayerShell,
}

impl App {
    fn find_widget<'a>(
        widgets: &'a mut Vec<Box<dyn Widget>>,
        layer_surface: &wl_surface::WlSurface,
    ) -> Option<&'a mut dyn Widget> {
        for widget in widgets {
            if layer_surface == widget.layer_surface().wl_surface() {
                return Some(widget.as_mut());
            }
        }
        None
    }

    fn find_widget_index(&self, layer_surface: &wl_surface::WlSurface) -> Option<usize> {
        return self
            .widgets
            .iter()
            .position(|w| w.layer_surface().wl_surface() == layer_surface);
    }

    fn focused_widget(&mut self) -> Option<&mut Box<dyn Widget>> {
        if self.selected_widget.is_some() {
            self.widgets.get_mut(self.selected_widget.unwrap())
        } else {
            None
        }
    }

    fn draw_widget(&mut self, layer_surface: &wl_surface::WlSurface) {
        if let Some(widget) = Self::find_widget(&mut self.widgets, layer_surface) {
            if let Some(ref mut painter) = self.painter {
                widget.draw_widget(&self.egl_data, painter);
            }
        }
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.should_exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer_surface: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(widget) = Self::find_widget(&mut self.widgets, layer_surface.wl_surface()) {
            widget.set_size(configure.new_size.0, configure.new_size.1);

            let (width, height) = widget.size();

            if widget.wl_egl_surface().is_none() {
                let wl_egl_surface =
                    WlEglSurface::new(layer_surface.wl_surface().id(), width as i32, height as i32)
                        .expect("Failed to create WL EGL Surface");

                let egl_surface = unsafe {
                    self.egl_data
                        .egl_instance
                        .create_platform_window_surface(
                            self.egl_data.egl_display,
                            self.egl_data.egl_config,
                            wl_egl_surface.ptr() as *mut std::ffi::c_void,
                            &[egl::ATTRIB_NONE],
                        )
                        .expect("Failed to create EGL Surface")
                };

                widget.set_wl_egl_surface(wl_egl_surface);
                widget.set_egl_surface(egl_surface);

                self.egl_data
                    .egl_instance
                    .make_current(
                        self.egl_data.egl_display,
                        Some(egl_surface),
                        Some(egl_surface),
                        Some(self.egl_data.egl_context),
                    )
                    .expect("Failed to set current");

                if self.egl_data.gl.is_none() {
                    self.egl_data.gl = Some(Arc::new(unsafe {
                        glow::Context::from_loader_function(|s| {
                            self.egl_data
                                .egl_instance
                                .get_proc_address(s)
                                .map(|f| f as *const std::ffi::c_void)
                                .unwrap_or(std::ptr::null())
                        })
                    }));

                    self.painter = Some(
                        egui_glow::Painter::new(
                            Arc::clone(self.egl_data.gl.as_ref().unwrap()),
                            "",
                            None,
                            false,
                        )
                        .expect("Failed to create egui painter"),
                    );
                }
            }
        }
        self.draw_widget(layer_surface.wl_surface());
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer_surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw_widget(layer_surface);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Pointer {
            _ = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to get pointer");
        } else if capability == Capability::Keyboard {
            _ = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to get keyboard");
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }

    fn remove_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl PointerHandler for App {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            if let Some(widget) = Self::find_widget(&mut self.widgets, &event.surface) {
                widget.handle_pointer_event(event);
            }
            self.draw_widget(&event.surface);
        }
    }
}

impl KeyboardHandler for App {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym],
    ) {
        self.selected_widget = self.find_widget_index(surface);
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _serial: u32,
    ) {
        if self.selected_widget == self.find_widget_index(surface) {
            self.selected_widget = None;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
        if let Some(widget) = self.focused_widget() {
            widget.handle_keyboard_event(&event);
        }
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: smithay_client_toolkit::seat::keyboard::Modifiers,
        _raw_modifiers: smithay_client_toolkit::seat::keyboard::RawModifiers,
        _layout: u32,
    ) {
        if let Some(widget) = self.focused_widget() {
            widget.handle_keyboard_modifiers(&modifiers);
        }
    }
}

delegate_compositor!(App);
delegate_layer!(App);
delegate_output!(App);
delegate_registry!(App);
delegate_pointer!(App);
delegate_seat!(App);
delegate_keyboard!(App);
