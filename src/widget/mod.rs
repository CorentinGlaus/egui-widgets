use smithay_client_toolkit::{
    reexports::client::protocol::wl_shm,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::slot::{Buffer, SlotPool},
};

pub trait Widget {
    fn layer_surface(&self) -> &LayerSurface;
    fn size(&self) -> (u32, u32);
    fn set_size(&mut self, width: u32, height: u32);
    fn render(&self, canvas: &mut [u8]);

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

pub struct TopBar {
    pub layer_surface: LayerSurface,
    pub buffer: Option<Buffer>,
    pub width: u32,
    pub height: u32,
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
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 180;
            pixel[1] = 180;
            pixel[2] = 0;
            pixel[3] = 255;
        }
    }

    fn store_buffer(&mut self, buffer: Buffer) {
        self.buffer = Some(buffer);
    }
}
