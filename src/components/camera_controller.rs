use glam::{Mat4, Vec3};
use crate::rendering::camera::Camera;
use crate::components::input_handler::CameraLockTarget;

const PITCH_MIN: f32 = -std::f32::consts::FRAC_PI_2 + 0.01;
const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub struct CameraController {
    pub camera: Camera,
    pub lock_target: CameraLockTarget,
    pub lock_offset: Vec3,
}

impl CameraController {
    pub fn new(initial_position: Vec3) -> Self {
        let mut camera = Camera::new(initial_position);
        camera.look_at(Vec3::ZERO);
        Self {
            camera,
            lock_target: CameraLockTarget::None,
            lock_offset: Vec3::new(0.0, 2.0, 5.0),
        }
    }
    
    pub fn process_input(&mut self, camera_input: &crate::components::input_handler::CameraInputState) {
        // Handle camera reset
        if camera_input.reset_camera {
            self.reset_camera();
            return;
        }
        
        // Handle camera lock changes
        if camera_input.lock_target != CameraLockTarget::None {
            self.lock_target = camera_input.lock_target;
            match self.lock_target {
                CameraLockTarget::Earth => self.lock_offset = Vec3::new(0.0, 0.5, 1.0),
                CameraLockTarget::Moon => self.lock_offset = Vec3::new(0.0, 0.5, 1.0),
                CameraLockTarget::Mars => self.lock_offset = Vec3::new(0.0, 0.5, 1.0),
                CameraLockTarget::Mercury => self.lock_offset = Vec3::new(0.0, 0.5, 1.0),
                CameraLockTarget::Venus => self.lock_offset = Vec3::new(0.0, 0.5, 1.0),
                _ => {}
            }
        } else if camera_input.lock_target == CameraLockTarget::None && self.lock_target != CameraLockTarget::None {
            self.lock_target = CameraLockTarget::None;
        }
        
        // Handle camera rotation
        self.camera.yaw += camera_input.rotation.x;
        self.camera.pitch -= camera_input.rotation.y;
        self.camera.clamp_pitch();
        
        // Handle movement or offset adjustment
        if self.lock_target == CameraLockTarget::None {
            // Free camera movement
            if camera_input.movement.length_squared() > 0.0 {
                let movement = camera_input.movement.normalize() * 0.016; // Using fixed delta time
                self.camera.position += movement;
            }
        } else {
            // Adjust offset when locked to a target
            self.lock_offset += camera_input.lock_offset_delta * 0.016;
        }
    }
    
    pub fn update_locked_camera(&mut self, target_position: Vec3) {
        if self.lock_target != CameraLockTarget::None {
            self.camera.position = target_position + self.lock_offset;
            self.camera.look_at(target_position);
        }
    }
    
    pub fn reset_camera(&mut self) {
        self.camera.position = Vec3::new(0.0, 2.0, 5.0);
        self.camera.look_at(Vec3::ZERO);
        self.lock_target = CameraLockTarget::None;
    }
    
    pub fn view_matrix(&self) -> Mat4 {
        self.camera.view_matrix()
    }
}