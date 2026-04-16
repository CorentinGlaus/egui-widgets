pub mod top_bar;

use smithay_client_toolkit::{
    reexports::client::protocol::wl_shm,
    seat::pointer::{PointerEvent, PointerEventKind},
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::slot::{Buffer, SlotPool},
};

pub trait Widget {
    fn layer_surface(&self) -> &LayerSurface;
    fn size(&self) -> (u32, u32);
    fn set_size(&mut self, width: u32, height: u32);
    fn render(&self, canvas: &mut [u8]);

    /// Handles a pointer event and returns whether a redraw is needed.
    fn handle_pointer_event(&mut self, pointer_event: &PointerEvent) -> bool;

    fn draw(&mut self, pool: &mut SlotPool) {
        let (width, height) = self.size();
        if width == 0 || height == 0 {
            return;
        }

        // bytes per row, ARGB -> 4
        let stride = width * 4;
        let (buffer, canvas) = pool
            .create_buffer(
                width as i32,
                height as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        self.render(canvas);

        let surface = self.layer_surface();
        let wl_surface = surface.wl_surface();

        wl_surface.attach(Some(&buffer.wl_buffer()), 0, 0);
        wl_surface.damage_buffer(0, 0, width as i32, height as i32);
        wl_surface.commit();

        self.store_buffer(buffer);
    }

    fn store_buffer(&mut self, buffer: Buffer);
}
