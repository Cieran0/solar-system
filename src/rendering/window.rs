// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent};
use std::error::Error;

// Simplified interface for OpenGL.
pub struct Window {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
}

impl Window {
    // Creates a new OpenGL 4.6 core profile window.
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, Box<dyn Error>> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
        glfw.window_hint(glfw::WindowHint::Floating(true));

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or("Failed to create GLFW window")?;

        window.make_current();
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol).unwrap() as *const _);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::DITHER);
        }

        Ok(Self { glfw, window, events })
    }

    // Returns if the window should be closed.
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    // Sets the window's close flag, typically used to request graceful shutdown.
    pub fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
    }

    // Get all pending GLFW events.
    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    // Swaps the front and back buffers.
    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    // Returns the current width of the window area.
    pub fn get_width(&self) -> u32 {
        self.window.get_size().0 as u32
    }

    // Returns the current height of the window area.
    pub fn get_height(&self) -> u32 {
        self.window.get_size().1 as u32
    }
}