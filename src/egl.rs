use khronos_egl as egl;
use smithay_client_toolkit::reexports::client::Connection;

pub struct EglData {
    pub egl_instance: egl::DynamicInstance<egl::EGL1_5>,
    pub egl_display: egl::Display,
    pub egl_config: egl::Config,
    pub egl_context: egl::Context,
    pub gl: Option<glow::Context>,
}

impl EglData {
    pub fn new(conn: &Connection) -> EglData {
        let wayland_display = conn.backend().display_ptr() as *mut std::ffi::c_void;

        let egl_instance = unsafe {
            egl::DynamicInstance::<egl::EGL1_5>::load_required().expect("Failed to load EGL")
        };

        let egl_display = unsafe {
            egl_instance
                .get_display(wayland_display)
                .expect("Failed to get EGL display")
        };

        egl_instance
            .initialize(egl_display)
            .expect("Failed to initialize EGL");

        let config_attribs = [
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::NONE,
        ];

        let egl_config = egl_instance
            .choose_first_config(egl_display, &config_attribs)
            .expect("Failed to choose config")
            .expect("No matching config found");

        let context_attribs = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];

        egl_instance
            .bind_api(egl::OPENGL_ES_API)
            .expect("Failed to bind OpenGL ES API");

        let egl_context = egl_instance
            .create_context(egl_display, egl_config, None, &context_attribs)
            .expect("Failed to create EGL context");

        EglData {
            egl_instance,
            egl_display,
            egl_config,
            egl_context,
            gl: None,
        }
    }
}
