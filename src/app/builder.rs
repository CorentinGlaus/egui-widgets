use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    shell::wlr_layer::{LayerShell, LayerSurface},
    shm::{
        Shm,
        slot::{Buffer, SlotPool},
    },
};

use crate::app::App;

pub struct AppBuilder {
    pub registry_state: RegistryState,
    pub compositor_state: CompositorState,
    pub output_state: OutputState,
    pub layer_shell: LayerShell,
    pub shm: Shm,

    pub layer_surface: Option<LayerSurface>,
    pub buffer: Option<Buffer>,
    pub pool: SlotPool,

    pub width: u32,
    pub height: u32,
    pub configured: bool,
    pub should_exit: bool,
}

impl AppBuilder {
    pub fn new(
        registry_state: RegistryState,
        compositor_state: CompositorState,
        output_state: OutputState,
        layer_shell: LayerShell,
        shm: Shm,
        slot_pool: SlotPool,
    ) -> AppBuilder {
        AppBuilder {
            registry_state,
            compositor_state,
            output_state,
            layer_shell,
            shm,
            layer_surface: None,
            buffer: None,
            pool: slot_pool,
            width: 0,
            height: 0,
            configured: false,
            should_exit: false,
        }
    }

    pub fn layer_surface(mut self, layer_surface: Option<LayerSurface>) -> AppBuilder {
        self.layer_surface = layer_surface;
        self
    }

    pub fn build(self) -> App {
        App {
            registry_state: self.registry_state,
            compositor_state: self.compositor_state,
            output_state: self.output_state,
            layer_shell: self.layer_shell,
            shm: self.shm,
            layer_surface: self.layer_surface,
            buffer: self.buffer,
            pool: self.pool,
            width: self.width,
            height: self.height,
            configured: self.configured,
            should_exit: self.should_exit,
        }
    }
}
