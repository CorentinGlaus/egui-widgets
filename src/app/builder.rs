use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell,
};

use crate::{app::App, egl::EglData, widget::Widget};

pub struct AppBuilder {
    pub registry_state: RegistryState,
    pub compositor_state: CompositorState,
    pub output_state: OutputState,
    pub layer_shell: LayerShell,
    pub seat_state: SeatState,

    pub egl_data: EglData,

    pub widgets: Vec<Box<dyn Widget>>,

    pub should_exit: bool,
}

impl AppBuilder {
    pub fn new(
        registry_state: RegistryState,
        compositor_state: CompositorState,
        output_state: OutputState,
        layer_shell: LayerShell,
        seat_state: SeatState,
        egl_data: EglData,
    ) -> AppBuilder {
        AppBuilder {
            registry_state,
            compositor_state,
            output_state,
            layer_shell,
            seat_state,
            egl_data,
            widgets: Vec::new(),
            should_exit: false,
        }
    }

    pub fn add_widget(mut self, widget: Box<dyn Widget>) -> AppBuilder {
        self.widgets.push(widget);
        self
    }

    pub fn build(self) -> App {
        App {
            registry_state: self.registry_state,
            _compositor_state: self.compositor_state,
            output_state: self.output_state,
            _layer_shell: self.layer_shell,
            seat_state: self.seat_state,
            egl_data: self.egl_data,
            widgets: self.widgets,
            should_exit: self.should_exit,
        }
    }
}
