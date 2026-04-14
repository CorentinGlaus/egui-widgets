use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, QueueHandle,
        globals::registry_queue_init,
        protocol::{wl_output, wl_shm, wl_surface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
    },
    shm::{
        Shm, ShmHandler,
        slot::{Buffer, SlotPool},
    },
};

struct App {
    registry_state: RegistryState,
    compositor_state: CompositorState,
    output_state: OutputState,
    shm: Shm,
    layer_shell: LayerShell,

    layer_surface: Option<LayerSurface>,
    buffer: Option<Buffer>,
    pool: SlotPool,

    width: u32,
    height: u32,
    configured: bool,
    should_exit: bool,
}

impl App {
    fn draw(&mut self, _qh: &QueueHandle<App>) {
        if !self.configured {
            return;
        }

        let surface = self.layer_surface.as_ref().unwrap();
        let width = self.width;
        let height = self.height;

        // bytes per row, ARGB -> 4
        let stride = width * 4;
        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 180;
            pixel[1] = 180;
            pixel[2] = 0;
            pixel[3] = 255;
        }

        let wl_surface = surface.wl_surface();

        wl_surface.attach(Some(&buffer.wl_buffer()), 0, 0);
        wl_surface.damage_buffer(0, 0, width as i32, height as i32);
        wl_surface.commit();

        self.buffer = Some(buffer);
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.should_exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0;
        self.height = configure.new_size.1;
        self.configured = true;

        self.draw(qh);
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
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
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

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}

delegate_compositor!(App);
delegate_layer!(App);
delegate_output!(App);
delegate_registry!(App);
delegate_shm!(App);

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let (globals, mut event_queue) =
        registry_queue_init::<App>(&conn).expect("Failed to init registry");

    let qh = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh).expect("Compositor not available");

    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");

    let shm = Shm::bind(&globals, &qh).expect("Shm not available");

    let pool = SlotPool::new(1024, &shm).expect("Failed to create SHM pool");

    let surface = compositor_state.create_surface(&qh);

    // TODO: Change namespace of layer surface
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("demo-panel"), None);

    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, 40);

    layer_surface.set_exclusive_zone(40);
    layer_surface.set_keyboard_interactivity(
        smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
    );

    layer_surface.commit();

    let mut app = App {
        registry_state: RegistryState::new(&globals),
        compositor_state,
        output_state: OutputState::new(&globals, &qh),
        shm,
        layer_shell,
        layer_surface: Some(layer_surface),
        buffer: None,
        pool,
        width: 0,
        height: 0,
        configured: false,
        should_exit: false,
    };

    while !app.should_exit {
        event_queue
            .blocking_dispatch(&mut app)
            .expect("Dispatch failed");
    }
}
