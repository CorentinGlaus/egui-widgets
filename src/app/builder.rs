use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerShell,
    shm::{Shm, slot::SlotPool},
};

use crate::{app::App, widget::Widget};

pub struct AppBuilder {
    pub registry_state: RegistryState,
    pub compositor_state: CompositorState,
    pub output_state: OutputState,
    pub layer_shell: LayerShell,
    pub seat_state: SeatState,
    pub shm: Shm,

    pub widgets: Vec<Box<dyn Widget>>,
    pub pool: SlotPool,

    pub should_exit: bool,
}

impl AppBuilder {
    pub fn new(
        registry_state: RegistryState,
        compositor_state: CompositorState,
        output_state: OutputState,
        layer_shell: LayerShell,
        seat_state: SeatState,
        shm: Shm,
        slot_pool: SlotPool,
    ) -> AppBuilder {
        AppBuilder {
            registry_state,
            compositor_state,
            output_state,
            layer_shell,
            seat_state,
            shm,
            widgets: Vec::new(),
            pool: slot_pool,
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
            compositor_state: self.compositor_state,
            output_state: self.output_state,
            layer_shell: self.layer_shell,
            seat_state: self.seat_state,
            shm: self.shm,
            widgets: self.widgets,
            pool: self.pool,
            should_exit: self.should_exit,
        }
    }
}
