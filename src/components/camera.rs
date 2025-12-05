// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use glam::{Mat4, Vec3};
use crate::components::input_handler::{CameraInputState, CameraLockTarget};

const PITCH_MIN: f32 = -std::f32::consts::FRAC_PI_2 + 0.01;
const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

// Represents a 3D camera with position, orientation, and optional target-locking behavior.
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vec3,
    pub lock_target: CameraLockTarget,
    pub lock_offset: Vec3,
}

impl Camera {
    // Creates a new camera at the given position and orients it to look at the origin.
    pub fn new(position: Vec3) -> Self {
        let mut cam = Self {
            position,
            yaw: 0.0, 
            pitch: 0.0,
            up: Vec3::Y,
            lock_target: CameraLockTarget::None,
            lock_offset: Vec3::splat(0.0f32),
        };
        cam.look_at(Vec3::splat(0.0f32));
        cam
    }

    // Updates camera state based on user input, including movement, rotation, locking, and reset.
    pub fn process_input(&mut self, camera_input: &CameraInputState) {
        // Handle camera reset
        if camera_input.reset_camera {
            self.reset_camera();
            return;
        }

        if camera_input.lock_target == CameraLockTarget::Clear {
            self.lock_target = CameraLockTarget::None;
        } else if camera_input.lock_target != CameraLockTarget::None {
            self.lock_target = camera_input.lock_target;
            self.lock_offset = Vec3::new(0.0, 0.5, 1.0);
        }

        // Handle camera rotation
        self.yaw += camera_input.rotation.x;
        self.pitch -= camera_input.rotation.y;
        self.clamp_pitch();

        // Free camera movement (only when NOT locked)
        if camera_input.movement.length_squared() > 0.0 {
            let forward = self.forward().normalize();
            let right = forward.cross(self.up).normalize();
            let world_up = Vec3::Y;
            let mut move_vec = Vec3::ZERO;
            move_vec += forward * camera_input.movement.z;
            move_vec += right * camera_input.movement.x;
            move_vec += world_up * camera_input.movement.y;
            let speed = 2.0 * 0.016;
            let detla_move = move_vec.normalize() * speed * camera_input.movement.length();

            if self.lock_target != CameraLockTarget::None {
                self.lock_offset += detla_move;
            } else {
                self.position += detla_move;
            }
        }
    }

    // Updates the camera's position when locked to a moving target.
    pub fn update_locked_camera(&mut self, target_position: Vec3) {
        if self.lock_target != CameraLockTarget::None {
            self.position = target_position + self.lock_offset;
        }
    }

    // Resets the camera to a default position and orientation, and disables target locking.
    pub fn reset_camera(&mut self) {
        self.position = Vec3::new(0.0, 2.0, 5.0);
        self.look_at(Vec3::ZERO);
        self.lock_target = CameraLockTarget::None;
    }

    // Computes and returns the normalized forward direction vector of the camera.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    // Computes and returns the camera's view matrix for rendering.
    pub fn view_matrix(&self) -> Mat4 {
        let target = self.position + self.forward();
        Mat4::look_at_rh(self.position, target, self.up)
    }

    // Clamps the camera's pitch angle to prevent gimbal lock and unnatural flips.
    pub fn clamp_pitch(&mut self) {
        self.pitch = self.pitch.clamp(PITCH_MIN, PITCH_MAX);
    }

    // Orients the camera to look at a specified target point in world space.
    pub fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalize();
        self.yaw = direction.x.atan2(direction.z);
        self.pitch = direction.y.asin();
        self.clamp_pitch();
    }
}