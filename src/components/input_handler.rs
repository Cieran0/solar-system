// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::collections::HashMap;
use glfw::{Action, Key, MouseButton, WindowEvent};
use glam::{Vec2, Vec3};
use crate::rendering::window::Window;

// Represents possible celestial bodies or states the camera can lock onto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraLockTarget {
    None,
    Earth,
    Moon,
    Mars,
    Mercury,
    Venus,
    Clear,
}

// Holds processed input state relevant to camera control.
#[derive(Debug, Clone, Copy)]
pub struct CameraInputState {
    pub movement: Vec3,
    pub rotation: Vec2,
    pub lock_target: CameraLockTarget,
    pub reset_camera: bool,
}

// Holds processed input state relevant to simulation controls.
#[derive(Debug, Clone, Copy)]
pub struct SimulationInputState {
    pub time_scale_multiplier: f32,
    pub spacecraft_rotation: Vec2,
    pub spacecraft_distance_delta: f32,
    pub framebuffer_resized: Option<(i32, i32)>,
}

// Manages raw input from GLFW and translates it into high-level camera and simulation states.
pub struct InputHandler {
    window: Window,
    keys: HashMap<Key, bool>,
    mouse_buttons: HashMap<MouseButton, bool>,
    cursor_position: Option<(f64, f64)>,
    last_cursor_position: Option<(f64, f64)>,
    last_frame_keys: HashMap<Key, bool>,
    framebuffer_resized: Option<(i32, i32)>,
}

// Provides a default value for CameraLockTarget (None).
impl Default for CameraLockTarget {
    fn default() -> Self {
        CameraLockTarget::None
    }
}

impl InputHandler {
    // Creates a new InputHandler associated with the given window.
    pub fn new(window: Window) -> Self {
        Self {
            window,
            keys: HashMap::new(),
            mouse_buttons: HashMap::new(),
            cursor_position: None,
            last_cursor_position: None,
            last_frame_keys: HashMap::new(),
            framebuffer_resized: None,
        }
    }

    // Polls and processes all input events, returning updated camera and simulation states.
    pub fn process_input(&mut self) -> (CameraInputState, SimulationInputState) {
        // Poll events and update internal state
        self.poll_events();
        
        let camera_state = self.process_camera_input();
        let simulation_state = self.process_simulation_input();
        
        // Save current key states for next frame
        self.last_frame_keys = self.keys.clone();
        
        (camera_state, simulation_state)
    }
    
    // Polls GLFW events and updates internal input state (keys, mouse, cursor, resize).
    fn poll_events(&mut self) {
        self.window.poll_events();
        self.last_cursor_position = self.cursor_position;
        let mut should_close = false;
        self.framebuffer_resized = None;

        for (_, event) in glfw::flush_messages(&self.window.events) {
            match event {
                WindowEvent::Key(key, _, action, _) => {
                    let pressed = action == Action::Press || action == Action::Repeat;
                    self.keys.insert(key, pressed);
                    
                    // Special handling for escape key
                    if key == Key::Escape && pressed {
                        should_close = true;
                    }
                }
                WindowEvent::MouseButton(button, action, _) => {
                    let pressed = action == Action::Press;
                    self.mouse_buttons.insert(button, pressed);
                }
                WindowEvent::CursorPos(xpos, ypos) => {
                    self.cursor_position = Some((xpos, ypos));
                }
                WindowEvent::FramebufferSize(width, height) => {
                    self.framebuffer_resized = Some((width, height));
                }
                _ => {}
            }
        }
        if should_close {
            self.window.set_should_close(should_close);
        }
    }
    
    // Returns true if the given key was pressed during the current frame (not held from before).
    fn key_pressed_this_frame(&self, key: Key) -> bool {
        *self.keys.get(&key).unwrap_or(&false) && !self.last_frame_keys.get(&key).unwrap_or(&false)
    }
    
    // Returns true if the given key is currently held down.
    fn is_key_down(&self, key: Key) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }
    
    // Returns true if the given mouse button is currently held down.
    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }
    
    // Computes and returns the change in cursor position since the last frame.
    fn get_mouse_delta(&self) -> Option<(f64, f64)> {
        if let (Some((x1, y1)), Some((x2, y2))) = (self.cursor_position, self.last_cursor_position) {
            Some((x1 - x2, y1 - y2))
        } else {
            None
        }
    }
    
    // Processes raw input into a structured camera input state.
    fn process_camera_input(&self) -> CameraInputState {
        let mut movement = Vec3::ZERO;
        let mut rotation = Vec2::ZERO;
        let mut lock_target = CameraLockTarget::None;
        let mut reset_camera = false;
        
        // Handle camera lock targets (pressed this frame)
        if self.key_pressed_this_frame(Key::Num0) { lock_target = CameraLockTarget::Clear; }
        if self.key_pressed_this_frame(Key::Num1) { reset_camera = true; }
        if self.key_pressed_this_frame(Key::Num2) { lock_target = CameraLockTarget::Earth; }
        if self.key_pressed_this_frame(Key::Num3) { lock_target = CameraLockTarget::Moon; }
        if self.key_pressed_this_frame(Key::Num4) { lock_target = CameraLockTarget::Mars; }
        if self.key_pressed_this_frame(Key::Num5) { lock_target = CameraLockTarget::Mercury; }
        if self.key_pressed_this_frame(Key::Num6) { lock_target = CameraLockTarget::Venus; }
        
        // Handle movement (held keys)
        let move_speed = 2.0;
        if self.is_key_down(Key::W) { movement.z += move_speed; }
        if self.is_key_down(Key::S) { movement.z -= move_speed; }
        if self.is_key_down(Key::A) { movement.x -= move_speed; }
        if self.is_key_down(Key::D) { movement.x += move_speed; }
        if self.is_key_down(Key::Space) { movement.y += move_speed; }
        if self.is_key_down(Key::LeftShift) { movement.y -= move_speed; }
        
        // Handle mouse rotation
        if self.is_mouse_button_down(MouseButton::Left) {
            if let Some((dx, dy)) = self.get_mouse_delta() {
                rotation = Vec2::new(dx as f32, dy as f32) * 0.001;
            }
        }
        
        CameraInputState {
            movement,
            rotation,
            lock_target,
            reset_camera,
        }
    }
    
    // Processes raw input into a structured simulation control state.
    fn process_simulation_input(&self) -> SimulationInputState {
        let mut time_scale_multiplier = 1.0;
        let mut spacecraft_rotation = Vec2::ZERO;
        let mut spacecraft_distance_delta = 0.0;
        
        // Handle simulation speed
        if self.is_key_down(Key::Equal) { time_scale_multiplier *= 1.2; }
        if self.is_key_down(Key::Minus) { time_scale_multiplier /= 1.2; }
        
        // Handle spacecraft control
        let rotation_speed = 1.5;
        if self.is_key_down(Key::Left) { spacecraft_rotation.x += rotation_speed; }
        if self.is_key_down(Key::Right) { spacecraft_rotation.x -= rotation_speed; }
        if self.is_key_down(Key::Up) { spacecraft_rotation.y += rotation_speed; }
        if self.is_key_down(Key::Down) { spacecraft_rotation.y -= rotation_speed; }
        
        // Handle spacecraft distance
        if self.is_key_down(Key::PageDown) { spacecraft_distance_delta -= 0.1; }
        if self.is_key_down(Key::PageUp) { spacecraft_distance_delta += 0.1; }

        SimulationInputState {
            time_scale_multiplier,
            spacecraft_rotation,
            spacecraft_distance_delta,
            framebuffer_resized: self.framebuffer_resized
        }
    }
    
    // Returns an immutable reference to the internal window.
    pub fn get_window(&self) -> &Window {
        &self.window
    }
    
    // Returns a mutable reference to the internal window.
    pub fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}