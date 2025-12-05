use std::collections::HashMap;
use glfw::{Action, Key, MouseButton, WindowEvent};
use glam::{Vec2, Vec3};
use crate::rendering::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraLockTarget {
    None,
    Sun,
    Earth,
    Moon,
    Mars,
    Mercury,
    Venus,
}

#[derive(Debug, Clone, Copy)]
pub struct CameraInputState {
    pub movement: Vec3,
    pub rotation: Vec2,
    pub lock_target: CameraLockTarget,
    pub lock_offset_delta: Vec3,
    pub reset_camera: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SimulationInputState {
    pub time_scale_multiplier: f32,
    pub spacecraft_rotation: Vec2,
    pub spacecraft_distance_delta: f32,
    pub should_close: bool,
    pub framebuffer_resized: Option<(i32, i32)>,
}

pub struct InputHandler {
    window: Window,
    keys: HashMap<Key, bool>,
    mouse_buttons: HashMap<MouseButton, bool>,
    cursor_position: Option<(f64, f64)>,
    last_cursor_position: Option<(f64, f64)>,
    cursor_captured: bool,
    last_frame_keys: HashMap<Key, bool>,
    framebuffer_resized: Option<(i32, i32)>,
}

impl Default for CameraLockTarget {
    fn default() -> Self {
        CameraLockTarget::None
    }
}

impl InputHandler {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            keys: HashMap::new(),
            mouse_buttons: HashMap::new(),
            cursor_position: None,
            last_cursor_position: None,
            cursor_captured: false,
            last_frame_keys: HashMap::new(),
            framebuffer_resized: None,
        }
    }

    pub fn process_input(&mut self) -> (CameraInputState, SimulationInputState) {
        // Poll events and update internal state
        self.poll_events();
        
        let camera_state = self.process_camera_input();
        let simulation_state = self.process_simulation_input();
        
        // Save current key states for next frame
        self.last_frame_keys = self.keys.clone();
        
        (camera_state, simulation_state)
    }
    
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
    
    fn key_pressed_this_frame(&self, key: Key) -> bool {
        *self.keys.get(&key).unwrap_or(&false) && !self.last_frame_keys.get(&key).unwrap_or(&false)
    }
    
    fn is_key_down(&self, key: Key) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }
    
    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }
    
    fn get_mouse_delta(&self) -> Option<(f64, f64)> {
        if let (Some((x1, y1)), Some((x2, y2))) = (self.cursor_position, self.last_cursor_position) {
            Some((x1 - x2, y1 - y2))
        } else {
            None
        }
    }
    
    fn process_camera_input(&self) -> CameraInputState {
        let mut movement = Vec3::ZERO;
        let mut rotation = Vec2::ZERO;
        let mut lock_target = CameraLockTarget::None;
        let mut lock_offset_delta = Vec3::ZERO;
        let mut reset_camera = false;
        
        // Handle camera lock targets (pressed this frame)
        if self.key_pressed_this_frame(Key::Num0) { lock_target = CameraLockTarget::None; }
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
        
        // Handle offset adjustment when locked
        if lock_target != CameraLockTarget::None {
            if self.is_key_down(Key::W) { lock_offset_delta.z += move_speed; }
            if self.is_key_down(Key::S) { lock_offset_delta.z -= move_speed; }
            if self.is_key_down(Key::A) { lock_offset_delta.x -= move_speed; }
            if self.is_key_down(Key::D) { lock_offset_delta.x += move_speed; }
            if self.is_key_down(Key::Space) { lock_offset_delta.y += move_speed; }
            if self.is_key_down(Key::LeftShift) { lock_offset_delta.y -= move_speed; }
            movement = Vec3::ZERO; // No regular movement when locked
        }
        
        CameraInputState {
            movement,
            rotation,
            lock_target,
            lock_offset_delta,
            reset_camera,
        }
    }
    
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
            should_close: self.window.should_close(),
            framebuffer_resized: self.framebuffer_resized
        }
    }
    
    pub fn get_window(&self) -> &Window {
        &self.window
    }
    
    pub fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}