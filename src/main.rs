mod app;
mod widget;

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{Connection, globals::registry_queue_init},
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};

use crate::{
    app::{App, builder::AppBuilder},
    widget::{Widget, top_bar::TopBar},
};

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let (globals, mut event_queue) =
        registry_queue_init::<App>(&conn).expect("Failed to init registry");

    let qh = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh).expect("Compositor not available");

    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");

    let seat_state = SeatState::new(&globals, &qh);

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

    let top_bar = TopBar {
        layer_surface: layer_surface,
        buffer: None,
        width: 0,
        height: 0,
        hovered: false,
    };

    let mut app = AppBuilder::new(
        RegistryState::new(&globals),
        compositor_state,
        OutputState::new(&globals, &qh),
        layer_shell,
        seat_state,
        shm,
        pool,
    )
    .add_widget(Box::new(top_bar))
    .build();

    while !app.should_exit {
        event_queue
            .blocking_dispatch(&mut app)
            .expect("Dispatch failed");
    }
}
