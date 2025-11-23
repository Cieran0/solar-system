use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent};
use std::error::Error;

pub struct Window {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, Box<dyn Error>> {
        // Initialise GLFW
        let mut glfw = glfw::init(glfw::fail_on_errors)?;

        // Setup OpenGL 4.6 Core
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

        // Floating for tiling window managers (doesn't work on bspwm for some reason, common issue with bspwm tho)
        glfw.window_hint(glfw::WindowHint::Floating(true));

        // Create the PWindow and event reciever, errors out on failure
        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or("Failed to create GLFW window")?;

        // Makes window current and polls for keyboard and mouse events
        window.make_current();
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol).unwrap() as *const _);

        // Enable Depth testing and dithering (makes it look slightly better)
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::DITHER);
        }

        Ok(Self { glfw, window, events })
    }

    // Check if the window should close
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    // Close the window
    pub fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
    }

    // Get all glfw windows
    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    // Get cursor position on window
    pub fn get_cursor_pos(&self) -> (f64, f64) {
        self.window.get_cursor_pos()
    }

    // Swap the window buffers to draw new frame
    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    // Not currently used as such _raw rather than raw, gets raw PWindow
    pub fn _raw(&mut self) -> &mut PWindow {
        &mut self.window
    }
}