// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::{collections::HashMap, rc::Rc};
use glam::{Mat4, Vec3};
use crate::{
    assets::{
        obj::ObjModel, qoi::QoiImage, shaders::{create_compute_program, create_shader_program}, shape::{Shape, Sphere}, texture
    }, components::{
        camera::Camera,
        input_handler::{CameraLockTarget, InputHandler},
        renderer::Renderer,
        scene::Scene,
    }, os_str, os_str_sub, rendering::{
        asteroid::AsteroidField,
        shadow::ShadowRenderer,
        skybox::Skybox,
        window::Window,
    }
};

// Holds the main simulation state including input, camera, scene, and renderer.
pub struct SolarSystem {
    input_handler: InputHandler,
    camera: Camera,
    scene: Scene,
    renderer: Renderer,
    sim_time_scale: f32,
}

// Describes an object to be rendered with geometry, transform, and material settings.
pub struct Renderable {
    pub geometry: Rc<dyn Shape>,
    pub model_matrix: Mat4,
    pub emit_mode: u32,
    pub use_texture: bool,
    pub texture_id: u32,
}

impl SolarSystem {
    // Initializes the solar system simulation with models, textures, shaders, and renderers.
    pub fn new(
        window: Window,
        images: HashMap<String, QoiImage>,
        asteroid_count: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let width = window.get_width() as i32;
        let height = window.get_height() as i32;

        // Input handler
        let input_handler = InputHandler::new(window);

        // Shaders
        let vertex_src = std::fs::read_to_string(os_str_sub("shaders", "render", "render.vert"))?;
        let fragment_src = std::fs::read_to_string(os_str_sub("shaders", "render", "render.frag"))?;
        let shader_program = create_shader_program(&vertex_src, &fragment_src)?;

        let instanced_vert = std::fs::read_to_string(os_str_sub("shaders", "asteroid", "instanced.vert"))?;
        let instanced_frag = std::fs::read_to_string(os_str_sub("shaders", "asteroid", "instanced.frag"))?;
        let instanced_shader_program = create_shader_program(&instanced_vert, &instanced_frag)?;

        // Textures
        let earth_texture = load_texture(&os_str_sub("textures", "planets", "earth.qoi"), &images)?;
        let sun_texture = load_texture(&os_str_sub("textures", "other_bodies", "sun.qoi"), &images)?;
        let moon_texture = load_texture(&os_str_sub("textures", "other_bodies", "moon.qoi"), &images)?;
        let mercury_texture = load_texture(&os_str_sub("textures", "planets", "mercury.qoi"), &images)?;
        let venus_texture = load_texture(&os_str_sub("textures", "planets", "venus.qoi"), &images)?;
        let mars_texture = load_texture(&os_str_sub("textures", "planets", "mars.qoi"), &images)?;
        let spacecraft_texture = load_texture(&os_str_sub("textures", "rocket", "rocket.qoi"), &images)?;
        let asteroid_texture = load_texture(&os_str_sub("textures", "other_bodies", "asteroid.qoi"), &images)?;

        // Skybox
        let skybox_faces = [
            &os_str_sub("textures", "background", "right.qoi"),
            &os_str_sub("textures", "background", "left.qoi"),
            &os_str_sub("textures", "background", "top.qoi"),
            &os_str_sub("textures", "background", "bottom.qoi"),
            &os_str_sub("textures", "background", "front.qoi"),
            &os_str_sub("textures", "background", "back.qoi"),
        ];
        let skybox_raws: [&QoiImage; 6] = skybox_faces
            .iter()
            .map(|path| images.get(*path).expect(&format!("Skybox texture missing: {}", path)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("Expected exactly 6 skybox faces");
        let skybox_texture = texture::create_cubemap_from_images(skybox_raws)?;
        let skybox = Skybox::new(skybox_texture)?;

        // Shadow renderer
        let shadow_renderer = ShadowRenderer::new(Vec3::ZERO, 4096)?;

        // Geometry
        let sun: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec3::new(1.0, 0.8, 0.0).extend(1.0)));
        let earth: Rc<dyn Shape> = Rc::new(ObjModel::new(&os_str("models", "earth.obj"))?);
        let moon: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec3::splat(0.5).extend(1.0)));
        let spacecraft: Rc<dyn Shape> = Rc::new(ObjModel::new(&os_str("models", "rocket.obj"))?);
        let mercury: Rc<dyn Shape> = Rc::new(Sphere::new(96, 96, Vec3::new(0.65, 0.57, 0.5).extend(1.0)));
        let venus: Rc<dyn Shape> = Rc::new(Sphere::new(120, 120, Vec3::new(1.0, 0.95, 0.75).extend(1.0)));
        let mars: Rc<dyn Shape> = Rc::new(Sphere::new(110, 110, Vec3::new(0.9, 0.4, 0.3).extend(1.0)));

        // Scene bodies
        let celestial_bodies: HashMap<String, crate::components::scene::CelestialBody> = [
            ("Sun".to_string(), crate::components::scene::CelestialBody::new(1.0, 0.0, 0.0, 0.1, 0.0, sun_texture, Rc::clone(&sun), 1)),
            ("Mercury".to_string(), crate::components::scene::CelestialBody::new(0.067, 3.0, 4.0, 1.0, 8.0_f32.to_radians(), mercury_texture, Rc::clone(&mercury), 0)),
            ("Venus".to_string(), crate::components::scene::CelestialBody::new(0.18, 5.0, 1.8, 0.5, 12_f32.to_radians(), venus_texture, Rc::clone(&venus), 0)),
            ("Earth".to_string(), crate::components::scene::CelestialBody::new(0.2, 7.0, 0.5, 2.0, 0.0_f32.to_radians(), earth_texture, Rc::clone(&earth), 0)),
            ("Moon".to_string(), crate::components::scene::CelestialBody::new(0.05, 0.4, 1.5, 1.5, 5.6_f32.to_radians(), moon_texture, Rc::clone(&moon), 0)),
            ("Mars".to_string(), crate::components::scene::CelestialBody::new(0.12, 10.0, 0.3, 1.8, -7_f32.to_radians(), mars_texture, Rc::clone(&mars), 0)),
        ].into_iter().collect();

        let scene = Scene::new(celestial_bodies, spacecraft, spacecraft_texture);

        // Asteroid field
        let compute_shader_source = std::fs::read_to_string(os_str_sub("shaders", "asteroid", "asteroid.glsl"))?;
        let compute_shader = create_compute_program(&compute_shader_source)?;

        let asteroid_field = AsteroidField::new(
            asteroid_count,
            &os_str("models", "asteroid.obj"),
            asteroid_texture,
            instanced_shader_program,
            compute_shader,
        )?;

        // Renderer and camera
        let renderer = Renderer::new(
            shader_program,
            instanced_shader_program,
            shadow_renderer,
            skybox,
            asteroid_field,
            width,
            height,
        );
        let camera = Camera::new(Vec3::new(0.0, 2.0, 5.0));

        Ok(Self {
            input_handler,
            camera,
            scene,
            renderer,
            sim_time_scale: 1.0,
        })
    }

    // Processes user input and updates simulation controls.
    pub fn handle_events(&mut self) {
        let (camera_input, sim_input) = self.input_handler.process_input();

        // Update camera with processed input
        self.camera.process_input(&camera_input);

        if let Some((width, height)) = sim_input.framebuffer_resized {
            self.renderer.resize(width, height);
        }

        // Apply simulation controls
        self.sim_time_scale *= sim_input.time_scale_multiplier;
        self.sim_time_scale = self.sim_time_scale.clamp(0.01, 10.0);

        // Spacecraft control
        self.scene.update_spacecraft(
            sim_input.spacecraft_rotation,
            sim_input.spacecraft_distance_delta,
        );

    }

    // Updates all simulation objects based on elapsed time.
    pub fn update(&mut self, delta_time: f32) {
        let dt = delta_time * self.sim_time_scale;

        // Update scene bodies
        self.scene.update(dt);

        // Update asteroids
        self.renderer.update_asteroids(dt);

        // Update camera position if locked
        if self.camera.lock_target != CameraLockTarget::None {
            let target_pos = self.scene.get_body_position(
                match self.camera.lock_target {
                    CameraLockTarget::Earth => "Earth",
                    CameraLockTarget::Moon => "Moon",
                    CameraLockTarget::Mars => "Mars",
                    CameraLockTarget::Mercury => "Mercury",
                    CameraLockTarget::Venus => "Venus",
                    CameraLockTarget::None => "Sun",
                    CameraLockTarget::Clear => "Sun",
                }
            );
            self.camera.update_locked_camera(target_pos);
        }
    }

    // Renders the current frame and swaps buffers.
    pub fn draw(&mut self) {
        let view = self.camera.view_matrix();
        let renderables = self.scene.get_renderables();
        self.renderer.draw(&view, &renderables);
        self.input_handler.get_window_mut().swap_buffers();
    }

    // Returns whether the application should close.
    pub fn should_close(&self) -> bool {
        self.input_handler.get_window().should_close()
    }

    // Cleans up GPU resources before shutdown.
    pub fn cleanup(&mut self) {
        self.renderer.cleanup();
    }
}

// Loads a texture from a pre-decoded QOI image.
fn load_texture(name: &str, images: &HashMap<String, QoiImage>) -> Result<u32, Box<dyn std::error::Error>> {
    let image = images.get(name).expect(&format!("{} texture missing", name));
    texture::create_texture_from_image(image)
}