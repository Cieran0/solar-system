// solar_system.rs
use std::{collections::HashMap, fs::read_to_string, path::Path, rc::Rc};
use gl::types::GLsizei;
use glam::{Mat4, Quat, Vec3, Vec4};
use glfw::{Action, Key, WindowEvent};
use rand::Rng;
use crate::{
    camera::Camera,
    obj::ObjModel,
    qoi::QoiImage,
    shape::{Shape, Sphere},
    shaders::create_shader_program,
    skybox::Skybox,
    shadow::ShadowRenderer,
    texture,
    transform_stack::TransformStack,
    uniforms::Uniforms,
    window::Window,
};

const MOVE_SPEED: f32 = 2.0;
const SPACECRAFT_SIZE: f32 = 0.00005;
const SPACECRAFT_ROT_SPEED: f32 = 1.5;
const MOUSE_SENSITIVITY: f32 = 0.001;

pub struct SolarSystem {
    camera: Camera,
    camera_lock_target: Option<String>,
    camera_lock_offset: Vec3,
    projection: Mat4,
    shader_program: u32,
    instanced_shader_program: u32,
    uniforms: Uniforms,
    instanced_uniforms: Uniforms,
    shadow_renderer: ShadowRenderer,
    skybox: Skybox,
    celestial_bodies: HashMap<String, CelestialBody>,
    asteroids: Vec<CelestialBody>,
    spacecraft: Rc<dyn Shape>,
    spacecraft_texture: u32,
    spacecraft_rotation: Quat,
    spacecraft_distance: f32,
    sim_time_scale: f32,
    win_width: i32,
    win_height: i32,
    keys_pressed: HashMap<Key, bool>,
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,
    asteroid_instance_vbo: u32,
    asteroid_model: ObjModel,
    window: Window,
}

pub struct Renderable {
    pub geometry: Rc<dyn Shape>,
    pub model_matrix: Mat4,
    pub emit_mode: u32,
    pub use_texture: bool,
    pub texture_id: u32,
}

pub struct CelestialBody {
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub texture: u32,
    pub geometry: Option<Rc<dyn Shape>>,
    pub rotation: f32,
    pub orbit_angle: f32,
    pub emit_mode: u32,
    pub inclination: f32,
}

impl CelestialBody {
    pub fn new(
        radius: f32,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
        inclination: f32,
        texture: u32,
        geometry: Option<Rc<dyn Shape>>,
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

    pub fn update(&mut self, dt: f32) {
        self.orbit_angle += self.orbit_speed * dt;
        self.rotation += self.rotation_speed * dt;
    }

    pub fn get_relative_position(&self) -> Vec3 {
        let x = self.orbit_radius * self.orbit_angle.cos();
        let mut z = self.orbit_radius * self.orbit_angle.sin();
        let y = z * self.inclination.sin();
        z = z * self.inclination.cos();
        Vec3::new(x, y, z)
    }
}

impl SolarSystem {
    pub fn new(window: Window, images: HashMap<String, QoiImage>) -> Result<Self, Box<dyn std::error::Error>> {
        let width = window.get_width() as i32;
        let height = window.get_height() as i32;
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }

        // Original shader
        let vertex_src = std::fs::read_to_string("shaders/render.vert")?;
        let fragment_src = std::fs::read_to_string("shaders/render.frag")?;
        let shader_program = create_shader_program(&vertex_src, &fragment_src)?;
        let uniforms = Uniforms::new(shader_program);

        // === SIMPLIFIED BUT FUNCTIONAL INSTANCED SHADERS ===
        let instanced_vert = read_to_string(os_str("shaders", "instanced.vert"))?;

        let instanced_frag = read_to_string(os_str("shaders", "instanced.frag"))?;

        let instanced_shader_program = create_shader_program(&instanced_vert, &instanced_frag)?;
        let instanced_uniforms = Uniforms::new(instanced_shader_program);

        let projection = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0,
            width as f32 / height as f32,
            0.1,
            100.0,
        );

        let earth_texture = load_texture(&os_str("textures", "earth.qoi"), &images)?;
        let sun_texture = load_texture(&os_str("textures", "sun.qoi"), &images)?;
        let moon_texture = load_texture(&os_str("textures", "moon.qoi"), &images)?;
        let mercury_texture = load_texture(&os_str("textures", "mercury.qoi"), &images)?;
        let venus_texture = load_texture(&os_str("textures", "venus.qoi"), &images)?;
        let mars_texture = load_texture(&os_str("textures", "mars.qoi"), &images)?;
        let spacecraft_texture = load_texture(&os_str("textures", "rocket.qoi"), &images)?;
        let asteroid_texture = load_texture(&os_str("textures", "astroid.qoi"), &images)?;

        let skybox_faces = [
            &os_str_sub("textures", "space", "right.qoi"),
            &os_str_sub("textures", "space", "left.qoi"),
            &os_str_sub("textures", "space", "top.qoi"),
            &os_str_sub("textures", "space", "bottom.qoi"),
            &os_str_sub("textures", "space", "front.qoi"),
            &os_str_sub("textures", "space", "back.qoi"),
        ];
        let skybox_raws: [&QoiImage; 6] = skybox_faces
            .iter()
            .map(|path| images.get(*path).expect(&format!("Skybox texture missing: {}", path)))
            .collect::<Vec<&QoiImage>>()
            .try_into()
            .expect("Incorrect number of skybox faces");
        let skybox_texture = texture::create_cubemap_from_images(skybox_raws)?;
        let skybox = Skybox::new(skybox_texture)?;

        let shadow_renderer = ShadowRenderer::new(Vec3::ZERO, 4096)?;

        let mut camera = Camera::new(Vec3::new(0.0, 2.0, 5.0));
        camera.look_at(Vec3::ZERO);

        let sun: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(1.0, 0.8, 0.0, 1.0)));
        let earth: Rc<dyn Shape> = Rc::new(ObjModel::new(&os_str("models", "earth.obj"))?);
        let moon: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(0.5, 0.5, 0.5, 1.0)));
        let spacecraft: Rc<dyn Shape> = Rc::new(ObjModel::new(&os_str("models", "rocket.obj"))?);
        let mercury: Rc<dyn Shape> = Rc::new(Sphere::new(96, 96, Vec4::new(0.65, 0.57, 0.5, 1.0)));
        let venus: Rc<dyn Shape> = Rc::new(Sphere::new(120, 120, Vec4::new(1.0, 0.95, 0.75, 1.0)));
        let mars: Rc<dyn Shape> = Rc::new(Sphere::new(110, 110, Vec4::new(0.9, 0.4, 0.3, 1.0)));
        let asteroid_obj = ObjModel::new(&os_str("models", "astroid_simple.obj"))?;

        let celestial_bodies: HashMap<String, CelestialBody> = [
            ("Sun".to_string(), CelestialBody::new(1.0, 0.0, 0.0, 0.1, 0.0, sun_texture, Some(Rc::clone(&sun)), 1)),
            ("Mercury".to_string(), CelestialBody::new(0.067, 3.0, 4.0, 1.0, 8.0_f32.to_radians(), mercury_texture, Some(Rc::clone(&mercury)), 0)),
            ("Venus".to_string(), CelestialBody::new(0.18, 5.0, 1.8, 0.5, 12_f32.to_radians(), venus_texture, Some(Rc::clone(&venus)), 0)),
            ("Earth".to_string(), CelestialBody::new(0.2, 7.0, 0.5, 2.0, 0.0_f32.to_radians(), earth_texture, Some(Rc::clone(&earth)), 0)),
            ("Moon".to_string(), CelestialBody::new(0.05, 0.4, 1.5, 1.5, 5.6_f32.to_radians(), moon_texture, Some(Rc::clone(&moon)), 0)),
            ("Mars".to_string(), CelestialBody::new(0.12, 10.0, 0.3, 1.8, -7_f32.to_radians(), mars_texture, Some(Rc::clone(&mars)), 0)),
        ].into_iter().collect();

        let mut asteroids = Vec::new();
        let asteroid_count = 10000;
        let mut rng = rand::rng();
        for _ in 0..asteroid_count {
            let orbit_radius = rng.random_range(11.0..14.0);
            let orbit_speed = rng.random_range(0.05..0.25);

            let orbit_angle = rng.random_range(-10.0..10.0 * std::f32::consts::PI);
            let inclination = (rng.random_range(-10.0..10.0) as f32).to_radians();
            let scale = rng.random_range(0.05..0.1);
            let mut asteroid_body = CelestialBody::new(
                scale,
                orbit_radius,
                orbit_speed,
                0.0,
                inclination,
                asteroid_texture,
                None,
                0,
            );
            asteroid_body.orbit_angle = orbit_angle;
            asteroids.push(asteroid_body);
        }

        // Instance buffer
        let mut asteroid_matrices: Vec<[[f32; 4]; 4]> = Vec::with_capacity(asteroid_count);
        for body in &asteroids {
            let pos = body.get_relative_position();
            let model = Mat4::from_translation(pos) * Mat4::from_scale(Vec3::splat(body.radius));
            asteroid_matrices.push(model.to_cols_array_2d());
        }
        let mut instance_vbo = 0;
        let instance_data_size = asteroid_count * std::mem::size_of::<[[f32; 4]; 4]>();
        unsafe {
            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                instance_data_size as isize,
                asteroid_matrices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        asteroid_obj.setup_instancing(instance_vbo);

        Ok(Self {
            window,
            camera,
            projection,
            shader_program,
            instanced_shader_program,
            uniforms,
            instanced_uniforms,
            shadow_renderer,
            skybox,
            celestial_bodies,
            asteroids,
            spacecraft,
            spacecraft_texture,
            spacecraft_rotation: Quat::IDENTITY,
            spacecraft_distance: 0.3,
            sim_time_scale: 1.0,
            win_width: width,
            win_height: height,
            keys_pressed: HashMap::new(),
            mouse_pressed: false,
            last_mouse_pos: None,
            camera_lock_target: None,
            camera_lock_offset: Vec3::ZERO,
            asteroid_instance_vbo: instance_vbo,
            asteroid_model: asteroid_obj,
        })
    }

    pub fn handle_events(&mut self) {
        self.window.poll_events();
        let events: Vec<_> = glfw::flush_messages(&self.window.events).collect();
        for (_, event) in events {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                WindowEvent::Key(key, _, Action::Press, _) => {
                    self.keys_pressed.insert(key, true);
                }
                WindowEvent::Key(key, _, Action::Release, _) => {
                    self.keys_pressed.insert(key, false);
                }
                WindowEvent::CursorPos(xpos, ypos) => {
                    if self.mouse_pressed {
                        if let Some((last_x, last_y)) = self.last_mouse_pos {
                            let dx = (xpos - last_x) as f32;
                            let dy = (ypos - last_y) as f32;
                            self.camera.yaw += dx * MOUSE_SENSITIVITY;
                            self.camera.pitch -= dy * MOUSE_SENSITIVITY;
                            self.camera.clamp_pitch();
                        }
                    }
                    self.last_mouse_pos = Some((xpos, ypos));
                }
                WindowEvent::MouseButton(glfw::MouseButton::Left, action, _) => {
                    self.mouse_pressed = action == Action::Press;
                    if self.mouse_pressed {
                        let (x, y) = self.window.get_cursor_pos();
                        self.last_mouse_pos = Some((x, y));
                    }
                }
                WindowEvent::FramebufferSize(width, height) => {
                    self.win_width = width;
                    self.win_height = height;
                    unsafe { gl::Viewport(0, 0, width, height); }
                    self.projection = Mat4::perspective_rh(
                        std::f32::consts::PI / 4.0,
                        width as f32 / height as f32,
                        0.1,
                        100.0,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let dt = delta_time * self.sim_time_scale;
        for body in self.celestial_bodies.values_mut() {
            body.update(dt);
        }
        for asteroid in self.asteroids.iter_mut() {
            asteroid.orbit_angle += asteroid.orbit_speed * dt;
        }

        let mut rot = Quat::IDENTITY;
        if *self.keys_pressed.get(&Key::Left).unwrap_or(&false) {
            rot = Quat::from_rotation_y(SPACECRAFT_ROT_SPEED * dt) * rot;
        }
        if *self.keys_pressed.get(&Key::Right).unwrap_or(&false) {
            rot = Quat::from_rotation_y(-SPACECRAFT_ROT_SPEED * dt) * rot;
        }
        if *self.keys_pressed.get(&Key::Up).unwrap_or(&false) {
            rot = rot * Quat::from_rotation_x(SPACECRAFT_ROT_SPEED * dt);
        }
        if *self.keys_pressed.get(&Key::Down).unwrap_or(&false) {
            rot = rot * Quat::from_rotation_x(-SPACECRAFT_ROT_SPEED * dt);
        }
        self.spacecraft_rotation = (rot * self.spacecraft_rotation).normalize();

        if *self.keys_pressed.get(&Key::PageDown).unwrap_or(&false) {
            self.spacecraft_distance = (self.spacecraft_distance - 0.1 * dt).max(0.05);
        }
        if *self.keys_pressed.get(&Key::PageUp).unwrap_or(&false) {
            self.spacecraft_distance += 0.1 * dt;
        }
        if *self.keys_pressed.get(&Key::Equal).unwrap_or(&false) {
            self.sim_time_scale *= 1.2;
        }
        if *self.keys_pressed.get(&Key::Minus).unwrap_or(&false) {
            self.sim_time_scale /= 1.2;
        }
        self.sim_time_scale = self.sim_time_scale.clamp(0.01, 10.0);

        if *self.keys_pressed.get(&Key::Num0).unwrap_or(&false) {
            self.camera_lock_target = None;
        }
        if *self.keys_pressed.get(&Key::Num1).unwrap_or(&false) {
            self.camera_lock_target = Some("Sun".to_string());
            self.camera.look_at(Vec3::ZERO);
        }
        if *self.keys_pressed.get(&Key::Num2).unwrap_or(&false) {
            let earth_pos = self.celestial_bodies["Earth"].get_relative_position();
            if self.camera_lock_target != Some("Earth".to_string()) {
                self.camera_lock_offset = Vec3::new(0.0, 0.5, 1.0);
            }
            self.camera_lock_target = Some("Earth".to_string());
            self.camera.look_at(earth_pos);
        }
        if *self.keys_pressed.get(&Key::Num3).unwrap_or(&false) {
            let earth_pos = self.celestial_bodies["Earth"].get_relative_position();
            let moon_pos = earth_pos + self.celestial_bodies["Moon"].get_relative_position();
            if self.camera_lock_target != Some("Moon".to_string()) {
                self.camera_lock_offset = Vec3::new(0.0, 0.5, 1.0);
            }
            self.camera_lock_target = Some("Moon".to_string());
            self.camera.look_at(moon_pos);
        }
        if *self.keys_pressed.get(&Key::Num4).unwrap_or(&false) {
            let mars_pos = self.celestial_bodies["Mars"].get_relative_position();
            if self.camera_lock_target != Some("Mars".to_string()) {
                self.camera_lock_offset = Vec3::new(0.0, 0.5, 1.0);
            }
            self.camera_lock_target = Some("Mars".to_string());
            self.camera.look_at(mars_pos);
        }
        if *self.keys_pressed.get(&Key::Num5).unwrap_or(&false) {
            let mercury_pos = self.celestial_bodies["Mercury"].get_relative_position();
            if self.camera_lock_target != Some("Mercury".to_string()) {
                self.camera_lock_offset = Vec3::new(0.0, 0.5, 1.0);
            }
            self.camera_lock_target = Some("Mercury".to_string());
            self.camera.look_at(mercury_pos);
        }
        if *self.keys_pressed.get(&Key::Num6).unwrap_or(&false) {
            let venus_pos = self.celestial_bodies["Venus"].get_relative_position();
            if self.camera_lock_target != Some("Venus".to_string()) {
                self.camera_lock_offset = Vec3::new(0.0, 0.5, 1.0);
            }
            self.camera_lock_target = Some("Venus".to_string());
            self.camera.look_at(venus_pos);
        }

        let fwd = self.camera.forward();
        let right = fwd.cross(self.camera.up).normalize();
        let mut vel = Vec3::ZERO;
        if *self.keys_pressed.get(&Key::W).unwrap_or(&false) { vel += fwd; }
        if *self.keys_pressed.get(&Key::S).unwrap_or(&false) { vel -= fwd; }
        if *self.keys_pressed.get(&Key::A).unwrap_or(&false) { vel -= right; }
        if *self.keys_pressed.get(&Key::D).unwrap_or(&false) { vel += right; }
        if *self.keys_pressed.get(&Key::Space).unwrap_or(&false) { vel += self.camera.up; }
        if *self.keys_pressed.get(&Key::LeftShift).unwrap_or(&false) { vel -= self.camera.up; }
        let movement = if vel.length_squared() > 0.0 {
            vel.normalize() * MOVE_SPEED * delta_time
        } else {
            Vec3::ZERO
        };
        if let Some(target_name) = &self.camera_lock_target {
            self.camera_lock_offset += movement;
            let body_pos = if target_name == "Sun" {
                Vec3::ZERO
            } else if target_name == "Moon" {
                self.celestial_bodies["Earth"].get_relative_position() +
                self.celestial_bodies[target_name].get_relative_position()
            } else {
                self.celestial_bodies[target_name].get_relative_position()
            };
            self.camera.position = body_pos + self.camera_lock_offset;
        } else {
            self.camera.position += movement;
        }
    }

    pub fn draw(&mut self) {
        let view = self.camera.view_matrix();
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let mut ts = TransformStack::new();
        let mut renderables = Vec::new();

        let sun = &self.celestial_bodies["Sun"];
        {
            let sun_model = ts.current()
                * Mat4::from_rotation_y(sun.rotation)
                * Mat4::from_scale(Vec3::splat(sun.radius));
            renderables.push(Renderable {
                geometry: Rc::clone(&sun.geometry.as_ref().expect("Sun model missing")),
                model_matrix: sun_model,
                emit_mode: sun.emit_mode,
                use_texture: true,
                texture_id: sun.texture,
            });
        }

        for planet_name in ["Mercury", "Venus", "Earth", "Mars"] {
            let planet = &self.celestial_bodies[planet_name];
            ts.push(Mat4::from_translation(planet.get_relative_position()));
            let planet_model = ts.current()
                * Mat4::from_rotation_y(planet.rotation)
                * Mat4::from_scale(Vec3::splat(planet.radius));
            renderables.push(Renderable {
                geometry: Rc::clone(&planet.geometry.as_ref().expect("Planet model missing")),
                model_matrix: planet_model,
                emit_mode: planet.emit_mode,
                use_texture: true,
                texture_id: planet.texture,
            });

            if planet_name == "Earth" {
                let moon = &self.celestial_bodies["Moon"];
                ts.push(Mat4::from_translation(moon.get_relative_position()));
                let moon_model = ts.current()
                    * Mat4::from_rotation_y(moon.rotation)
                    * Mat4::from_scale(Vec3::splat(moon.radius));
                renderables.push(Renderable {
                    geometry: Rc::clone(&moon.geometry.as_ref().expect("Moon model missing")),
                    model_matrix: moon_model,
                    emit_mode: moon.emit_mode,
                    use_texture: true,
                    texture_id: moon.texture,
                });

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
                ts.pop();
                ts.pop();
            }
            ts.pop();
        }

        // Shadow pass (without asteroids)
        self.shadow_renderer.render_depth_pass(&renderables);

        // === DRAW MAIN BODIES ===
        unsafe {
            gl::UseProgram(self.shader_program);
        }
        self.uniforms.set_view_matrix(&view);
        self.uniforms.set_projection_matrix(&self.projection);
        self.uniforms.set_light_pos(Vec4::new(0.0, 0.0, 0.0, 1.0));
        self.uniforms.set_light_pos_world(Vec3::ZERO);
        self.uniforms.set_shadow_map(1);
        self.uniforms.set_shadow_far(self.shadow_renderer.shadow_far);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.shadow_renderer.depth_cubemap);
            gl::Viewport(0, 0, self.win_width, self.win_height);
        }
        self.skybox.draw(&view, &self.projection);

        for r in &renderables {
            self.uniforms.set_model_matrix(&r.model_matrix);
            self.uniforms.set_normal_matrix(&r.model_matrix);
            self.uniforms.set_emit_mode(r.emit_mode);
            self.uniforms.set_use_texture(r.use_texture);
            if r.use_texture {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, r.texture_id);
                }
                self.uniforms.set_base_texture(0);
            }
            r.geometry.draw();
        }

        // === UPDATE ASTEROID INSTANCE BUFFER ===
        let mut matrices: Vec<[[f32; 4]; 4]> = Vec::with_capacity(self.asteroids.len());
        for asteroid in &self.asteroids {
            let pos = asteroid.get_relative_position();
            let model = Mat4::from_translation(pos) * Mat4::from_scale(Vec3::splat(asteroid.radius));
            matrices.push(model.to_cols_array_2d());
        }
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.asteroid_instance_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (matrices.len() * std::mem::size_of::<[[f32; 4]; 4]>()) as isize,
                matrices.as_ptr() as *const _,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // === DRAW ASTEROIDS (INSTANCED) ===
        unsafe {
            gl::UseProgram(self.instanced_shader_program);
        }
        self.instanced_uniforms.set_view_matrix(&view);
        self.instanced_uniforms.set_projection_matrix(&self.projection);
        self.instanced_uniforms.set_light_pos_world(Vec3::ZERO);
        self.instanced_uniforms.set_shadow_map(1);
        self.instanced_uniforms.set_shadow_far(self.shadow_renderer.shadow_far);
        self.instanced_uniforms.set_use_texture(true);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.asteroids[0].texture);
            self.instanced_uniforms.set_base_texture(0);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.shadow_renderer.depth_cubemap);
        }

        self.asteroid_model.draw_instanced(self.asteroids.len() as GLsizei);

        let error = unsafe { gl::GetError() };
        if error != gl::NO_ERROR {
            println!("OpenGL error after instanced draw: {}", error);
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    pub fn cleanup(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteProgram(self.instanced_shader_program);
            gl::DeleteBuffers(1, &self.asteroid_instance_vbo);
        }
    }
}

fn load_texture(name: &str, images: &HashMap<String, QoiImage>) -> Result<u32, Box<dyn std::error::Error>> {
    let image = images.get(name).expect(&format!("{} texture missing", name));
    texture::create_texture_from_image(image)
}

fn os_str(dir: &str, file: &str) -> String {
    Path::new(dir).join(file).to_str().unwrap().to_string()
}

fn os_str_sub(dir: &str, sub: &str, file: &str) -> String {
    Path::new(dir).join(sub).join(file).to_str().unwrap().to_string()
}