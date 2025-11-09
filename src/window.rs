use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, Window as GlfwWindow, WindowEvent};
use std::error::Error;

pub struct Window {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
}

impl Window {
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

        gl::load_with(|symbol| window.get_proc_address(symbol).unwrap() as *const _);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::DITHER);
        }

        Ok(Self { glfw, window, events })
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    pub fn get_cursor_pos(&self) -> (f64, f64) {
        self.window.get_cursor_pos()
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    pub fn _raw(&mut self) -> &mut GlfwWindow {
        &mut self.window
    }
}