// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::rc::Rc;
use std::collections::HashMap;
use glam::{Mat4, Quat, Vec3};
use crate::{
    assets::shape::Shape,
    transform_stack::TransformStack,
    simulation::solar_system::Renderable,
};

const SPACECRAFT_SIZE: f32 = 0.00005;

// Represents a planet, moon, or star with orbital and rotational properties.
pub struct CelestialBody {
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub texture: u32,
    pub geometry: Rc<dyn Shape>,
    pub rotation: f32,
    pub orbit_angle: f32,
    pub emit_mode: u32,
    pub inclination: f32,
}

impl CelestialBody {
    // Creates a new celestial body with the given physical and visual properties.
    pub fn new(
        radius: f32,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
        inclination: f32,
        texture: u32,
        geometry: Rc<dyn Shape>,
        emit_mode: u32,
    ) -> Self {
        Self {
            radius,
            orbit_radius,
            orbit_speed,
            rotation_speed,
            texture,
            geometry,
            rotation: 0.0,
            orbit_angle: 0.0,
            emit_mode,
            inclination,
        }
    }

    // Updates the orbital and rotational angles based on elapsed time.
    pub fn update(&mut self, dt: f32) {
        self.orbit_angle += self.orbit_speed * dt;
        self.rotation += self.rotation_speed * dt;
    }

    // Computes the body’s position relative to its parent (e.g., planet around the Sun).
    pub fn get_relative_position(&self) -> Vec3 {
        let x = self.orbit_radius * self.orbit_angle.cos();
        let mut z = self.orbit_radius * self.orbit_angle.sin();
        let y = z * self.inclination.sin();
        z = z * self.inclination.cos();
        Vec3::new(x, y, z)
    }
}

// Manages all celestial bodies and the player-controlled spacecraft in the simulation.
pub struct Scene {
    pub celestial_bodies: HashMap<String, CelestialBody>,
    pub spacecraft: Rc<dyn Shape>,
    pub spacecraft_texture: u32,
    pub spacecraft_rotation: Quat,
    pub spacecraft_distance: f32,
}

impl Scene {
    // Initializes the scene with celestial bodies and a spacecraft model.
    pub fn new(
        celestial_bodies: HashMap<String, CelestialBody>,
        spacecraft: Rc<dyn Shape>,
        spacecraft_texture: u32,
    ) -> Self {
        Self {
            celestial_bodies,
            spacecraft,
            spacecraft_texture,
            spacecraft_rotation: Quat::IDENTITY,
            spacecraft_distance: 0.3,
        }
    }

    // Updates all celestial bodies over time.
    pub fn update(&mut self, dt: f32) {
        for body in self.celestial_bodies.values_mut() {
            body.update(dt);
        }
    }

    // Updates the spacecraft’s orientation and distance from Earth based on user input.
    pub fn update_spacecraft(&mut self, rotation_delta: glam::Vec2, distance_delta: f32) {
        let dt = 0.016; // fixed timestep for consistent control feel
        if rotation_delta.length_squared() > 0.0 {
            let mut rot = Quat::IDENTITY;
            rot = Quat::from_rotation_y(rotation_delta.x * dt) * rot;
            rot = rot * Quat::from_rotation_x(rotation_delta.y * dt);
            self.spacecraft_rotation = (rot * self.spacecraft_rotation).normalize();
        }
        self.spacecraft_distance = (self.spacecraft_distance + distance_delta * dt).max(0.05);
    }

    // Generates a list of renderable objects with their model matrices and rendering parameters.
    pub fn get_renderables(&self) -> Vec<Renderable> {
        let mut renderables = Vec::new();
        let mut ts = TransformStack::new();

        // Sun
        let sun = &self.celestial_bodies["Sun"];
        let sun_model = ts.current()
            * Mat4::from_rotation_y(sun.rotation)
            * Mat4::from_scale(Vec3::splat(sun.radius));
        renderables.push(Renderable {
            geometry: Rc::clone(&sun.geometry),
            model_matrix: sun_model,
            emit_mode: sun.emit_mode,
            use_texture: true,
            texture_id: sun.texture,
        });

        // Planets
        for planet_name in ["Mercury", "Venus", "Earth", "Mars"] {
            let planet = &self.celestial_bodies[planet_name];
            ts.push(Mat4::from_translation(planet.get_relative_position()));
            let planet_model = ts.current()
                * Mat4::from_rotation_y(planet.rotation)
                * Mat4::from_scale(Vec3::splat(planet.radius));
            renderables.push(Renderable {
                geometry: Rc::clone(&planet.geometry),
                model_matrix: planet_model,
                emit_mode: planet.emit_mode,
                use_texture: true,
                texture_id: planet.texture,
            });

            // Moon & spacecraft (only for Earth)
            if planet_name == "Earth" {
                let moon = &self.celestial_bodies["Moon"];
                ts.push(Mat4::from_translation(moon.get_relative_position()));
                let moon_model = ts.current()
                    * Mat4::from_rotation_y(moon.rotation)
                    * Mat4::from_scale(Vec3::splat(moon.radius));
                renderables.push(Renderable {
                    geometry: Rc::clone(&moon.geometry),
                    model_matrix: moon_model,
                    emit_mode: moon.emit_mode,
                    use_texture: true,
                    texture_id: moon.texture,
                });

                // Spacecraft
                ts.push(Mat4::from_quat(self.spacecraft_rotation));
                ts.push(Mat4::from_translation(Vec3::new(0.0, 0.0, self.spacecraft_distance)));
                let spacecraft_model = ts.current() * Mat4::from_scale(Vec3::splat(SPACECRAFT_SIZE));
                renderables.push(Renderable {
                    geometry: Rc::clone(&self.spacecraft),
                    model_matrix: spacecraft_model,
                    emit_mode: 0,
                    use_texture: true,
                    texture_id: self.spacecraft_texture,
                });
                ts.pop(); // spacecraft
                ts.pop(); // spacecraft
                ts.pop(); // moon
            }
            ts.pop(); // planet
        }

        renderables
    }

    // Returns the current world position of a named celestial body.
    pub fn get_body_position(&self, name: &str) -> Vec3 {
        match name {
            "Sun" => Vec3::ZERO,
            "Moon" => {
                let earth_pos = self.celestial_bodies["Earth"].get_relative_position();
                earth_pos + self.celestial_bodies["Moon"].get_relative_position()
            }
            _ => self.celestial_bodies[name].get_relative_position(),
        }
    }
}