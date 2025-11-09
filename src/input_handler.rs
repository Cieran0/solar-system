use std::collections::HashSet;

use crate::camera::Camera;
use crate::window::Window;
use glfw::{Action, Key, WindowEvent};

const MOUSE_SENSITIVITY: f32 = 0.001;

pub struct InputHandler {
    pub keys_pressed: HashSet<Key>,
    pub mouse_pressed: bool,
    pub framebuffer_size: Option<(i32, i32)>,
    last_mouse_pos: Option<(f64, f64)>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_pressed: false,
            framebuffer_size: None,
            last_mouse_pos: None,
        }
    }

    pub fn handle_events(
        &mut self,
        gl_window: &mut Window,
        camera: &mut Camera,
    ) {
        let events: Vec<_> = glfw::flush_messages(&gl_window.events).collect();

        for (_, event) in events {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    gl_window.set_should_close(true);
                }
                WindowEvent::Key(key, _, Action::Press, _) => {
                    self.keys_pressed.insert(key);
                }
                WindowEvent::Key(key, _, Action::Release, _) => {
                    self.keys_pressed.remove(&key);
                }
                WindowEvent::CursorPos(xpos, ypos) => {
                    if self.mouse_pressed {
                        if let Some((last_x, last_y)) = self.last_mouse_pos {
                            let dx = (xpos - last_x) as f32;
                            let dy = (ypos - last_y) as f32;
                            camera.yaw += dx * MOUSE_SENSITIVITY;
                            camera.pitch -= dy * MOUSE_SENSITIVITY;
                            camera.clamp_pitch();
                        }
                    }
                    self.last_mouse_pos = Some((xpos, ypos));
                }
                WindowEvent::MouseButton(glfw::MouseButton::Left, action, _) => {
                    self.mouse_pressed = action == Action::Press;
                    if self.mouse_pressed {
                        let (x, y) = gl_window.get_cursor_pos();
                        self.last_mouse_pos = Some((x, y));
                    }
                }
                WindowEvent::FramebufferSize(width, height) => {
                    self.framebuffer_size = Some((width, height));
                }
                _ => {}
            }
        }
    }
}