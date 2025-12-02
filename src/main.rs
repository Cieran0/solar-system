mod window;
mod camera;
mod shaders;
mod transform_stack;
mod input_handler;
mod shape;
mod obj;
mod shadow;
mod skybox;
mod qoi;

use std::{
    collections::HashMap,
    error::Error,
    fs::read_to_string,
    rc::Rc,
    time::Instant,
};
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};

use crate::{
    camera::Camera, input_handler::InputHandler, obj::ObjModel, qoi::QoiDesc, shaders::create_shader_program, shadow::ShadowRenderer, shape::{Shape, Sphere}, skybox::Skybox, transform_stack::TransformStack, window::Window
};

struct Renderable {
    geometry: Rc<dyn Shape>,
    model_matrix: Mat4,
    emit_mode: u32,
    use_texture: bool,
    texture_id: u32,
}

struct CelestialBody {
    radius: f32,
    orbit_radius: f32, // Distance from parent
    orbit_speed: f32,
    rotation_speed: f32,
    texture: u32,
    geometry: Rc<dyn Shape>,
    rotation: f32,      // Spin rotation
    orbit_angle: f32,   // Angle around parent
    emit_mode: u32,
}

impl CelestialBody {
    fn new(
        radius: f32,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
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
        }
    }

    fn update(&mut self, dt: f32) {
        self.orbit_angle += self.orbit_speed * dt;
        self.rotation += self.rotation_speed * dt;
    }

    // Get relative position to parent (not absolute position)
    fn get_relative_position(&self) -> Vec3 {
        Vec3::new(
            self.orbit_radius * self.orbit_angle.cos(),
            0.0,
            self.orbit_radius * self.orbit_angle.sin(),
        )
    }
}

fn load_qoi_texture(path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let mut desc = QoiDesc { width: 0, height: 0, channels: 0, colorspace: 0 };
    let data = qoi::read(path, &mut desc, 4)?;

    let mut texture_id = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            desc.width as i32,
            desc.height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    Ok(texture_id)
}

fn load_skybox_textures(paths: [&str; 6]) -> Result<u32, Box<dyn std::error::Error>> {
    let mut cubemap_id = 0;
    unsafe {
        gl::GenTextures(1, &mut cubemap_id);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_id);

        // Define the faces in OpenGL cubemap order
        const CUBE_MAP_FACES: [u32; 6] = [
            gl::TEXTURE_CUBE_MAP_POSITIVE_X, // Right
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X, // Left
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y, // Top
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, // Bottom
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z, // Front
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, // Back
        ];

        for (i, path) in paths.iter().enumerate() {
            let mut desc = QoiDesc { width: 0, height: 0, channels: 0, colorspace: 0 };
            let data = qoi::read(path, &mut desc, 4)?;
            gl::TexImage2D(
                CUBE_MAP_FACES[i],
                0,
                gl::RGBA8 as i32,
                desc.width as i32,
                desc.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
        }

        // Set texture parameters
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

        gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
    }

    Ok(cubemap_id)
}

fn print_controls() {
    println!("=== Controls ===");
    println!("Camera movement: W/A/S/D + Space (up) / Left Shift (down)");
    println!("Camera views (Hold): 1 = Sun, 2 = Earth, 3 = Moon, 4 = Mars, 5 = Mercury, 6 = Venus");
    println!("Simulation speed: '=' = faster, '-' = slower");
    println!("Spacecraft rotation: Arrow keys");
    println!("Spacecraft distance from Moon: PageUp / PageDown");
    println!("================");
}

fn main() -> Result<(), Box<dyn Error>> {
    print_controls();

    let width = 1280i32;
    let height = 720i32;
    let mut win_width = width;
    let mut win_height = height;
    let mut window = Window::new(width as u32, height as u32, "Solar System")?;

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let vertex_src = read_to_string("shaders/render.vert")?;
    let fragment_src = read_to_string("shaders/render.frag")?;
    let shader_program = create_shader_program(&vertex_src, &fragment_src)?;
    unsafe { gl::UseProgram(shader_program) };


    let model_loc = unsafe { gl::GetUniformLocation(shader_program, b"model\0".as_ptr() as *const _) };
    let view_loc = unsafe { gl::GetUniformLocation(shader_program, b"view\0".as_ptr() as *const _) };
    let proj_loc = unsafe { gl::GetUniformLocation(shader_program, b"projection\0".as_ptr() as *const _) };
    let normal_matrix_loc = unsafe { gl::GetUniformLocation(shader_program, b"normal_matrix\0".as_ptr() as *const _) };
    let light_pos_loc = unsafe { gl::GetUniformLocation(shader_program, b"light_pos\0".as_ptr() as *const _) };
    let light_pos_world_loc = unsafe { gl::GetUniformLocation(shader_program, b"light_pos_world\0".as_ptr() as *const _) };
    let emit_mode_loc = unsafe { gl::GetUniformLocation(shader_program, b"emit_mode\0".as_ptr() as *const _) };
    let use_texture_loc = unsafe { gl::GetUniformLocation(shader_program, b"use_texture\0".as_ptr() as *const _) };
    let base_texture_loc = unsafe { gl::GetUniformLocation(shader_program, b"base_texture\0".as_ptr() as *const _) };
    let shadow_map_loc = unsafe { gl::GetUniformLocation(shader_program, b"shadow_map\0".as_ptr() as *const _) };
    let shadow_far_loc = unsafe { gl::GetUniformLocation(shader_program, b"shadow_far\0".as_ptr() as *const _) };

    let earth_texture = load_qoi_texture("textures/earth.qoi")?;
    let sun_texture = load_qoi_texture("textures/sun.qoi")?;
    let moon_texture = load_qoi_texture("textures/moon.qoi")?;
    let mercury_texture = load_qoi_texture("textures/mercury.qoi")?;
    let venus_texture = load_qoi_texture("textures/venus.qoi")?;
    let mars_texture = load_qoi_texture("textures/mars.qoi")?;
    let spacecraft_texture = load_qoi_texture("textures/rocket.qoi")?;

    let skybox_texture = load_skybox_textures(
        [
            "textures/space/right.qoi",
            "textures/space/left.qoi",
            "textures/space/top.qoi",
            "textures/space/bottom.qoi",
            "textures/space/front.qoi",
            "textures/space/back.qoi",
        ],
    )?;

    let skybox = Skybox::new(skybox_texture)?;

    let shadow_renderer = ShadowRenderer::new(Vec3::ZERO, 4096)?;

    let mut camera = Camera::new(Vec3::new(0.0, 2.0, 5.0));
    camera.look_at(Vec3::ZERO);
    let mut projection = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        win_width as f32 / win_height as f32,
        0.1,
        100.0,
    );

    let sun: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(1.0, 0.8, 0.0, 1.0)));
    let earth: Rc<dyn Shape> = Rc::new(ObjModel::new("models/earth.obj")?);
    let moon: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(0.5, 0.5, 0.5, 1.0)));
    let spacecraft: Rc<dyn Shape> = Rc::new(ObjModel::new("models/rocket.obj")?);
    let mercury: Rc<dyn Shape> = Rc::new(Sphere::new(96, 96, Vec4::new(0.65, 0.57, 0.5, 1.0)));
    let venus: Rc<dyn Shape> = Rc::new(Sphere::new(120, 120, Vec4::new(1.0, 0.95, 0.75, 1.0)));
    let mars: Rc<dyn Shape> = Rc::new(Sphere::new(110, 110, Vec4::new(0.9, 0.4, 0.3, 1.0)));

    let mut celestial_bodies: HashMap<String, CelestialBody> = [
        (
            "Sun".to_string(),
            CelestialBody::new(1.0, 0.0, 0.0, 0.1, sun_texture, Rc::clone(&sun), 1)
        ),
        (
            "Mercury".to_string(),
            CelestialBody::new(0.08, 3.0, 4.0, 1.0, mercury_texture, Rc::clone(&mercury), 0)
        ),
        (
            "Venus".to_string(),
            CelestialBody::new(0.18, 5.0, 1.8, 0.5, venus_texture, Rc::clone(&venus), 0)
        ),
        (
            "Earth".to_string(),
            CelestialBody::new(0.2, 7.0, 0.5, 2.0, earth_texture, Rc::clone(&earth), 0)
        ),
        (
            "Moon".to_string(),
            CelestialBody::new(0.06, 0.4, 1.5, 1.5, moon_texture, Rc::clone(&moon), 0)
        ),
        (
            "Mars".to_string(),
            CelestialBody::new(0.12, 10.0, 0.3, 1.8, mars_texture, Rc::clone(&mars), 0)
        ),
    ]
    .into_iter()
    .collect();

    const MOVE_SPEED: f32 = 2.0;
    const SPACECRAFT_SIZE: f32 = 0.00001;
    const SPACECRAFT_ROT_SPEED: f32 = 1.5;

    let mut spacecraft_rot = Quat::IDENTITY;
    let mut spacecraft_dist: f32 = 0.3;
    let mut sim_time_scale: f32 = 1.0;

    let mut input_handler = InputHandler::new();
    unsafe { gl::Viewport(0, 0, win_width, win_height); }
    let mut last_time = Instant::now();

    while !window.should_close() {
        let now = Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;

        window.poll_events();
        input_handler.handle_events(&mut window, &mut camera);

        if let Some((w, h)) = input_handler.framebuffer_size.take() {
            win_width = w;
            win_height = h;
            unsafe { gl::Viewport(0, 0, win_width, win_height); }
            projection = Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                win_width as f32 / win_height as f32,
                0.1,
                100.0,
            );
        }

        if input_handler.keys_pressed.contains(&glfw::Key::Equal) {
            sim_time_scale *= 1.2;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Minus) {
            sim_time_scale /= 1.2;
        }
        sim_time_scale = sim_time_scale.clamp(0.01, 10.0);
        let dt = delta * sim_time_scale;

        // Update all celestial bodies
        for body in celestial_bodies.values_mut() {
            body.update(dt);
        }

        let mut rot = Quat::IDENTITY;
        if input_handler.keys_pressed.contains(&glfw::Key::Left) {
            rot = Quat::from_rotation_y(SPACECRAFT_ROT_SPEED * dt) * rot;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Right) {
            rot = Quat::from_rotation_y(-SPACECRAFT_ROT_SPEED * dt) * rot;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Up) {
            rot = rot * Quat::from_rotation_x(SPACECRAFT_ROT_SPEED * dt);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Down) {
            rot = rot * Quat::from_rotation_x(-SPACECRAFT_ROT_SPEED * dt);
        }
        spacecraft_rot = (rot * spacecraft_rot).normalize();

        if input_handler.keys_pressed.contains(&glfw::Key::PageDown) {
            spacecraft_dist = (spacecraft_dist - 0.1 * dt).max(0.05);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::PageUp) {
            spacecraft_dist += 0.1 * dt;
        }

        // Camera view shortcuts using on-demand position calculation
        if input_handler.keys_pressed.contains(&glfw::Key::Num1) {
            camera.position = Vec3::new(0.0, 2.0, 5.0);
            camera.look_at(Vec3::ZERO);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num2) {
            let earth_pos = celestial_bodies["Earth"].get_relative_position();
            camera.position = earth_pos + Vec3::new(0.0, 0.5, 1.0);
            camera.look_at(earth_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num3) {
            let earth_pos = celestial_bodies["Earth"].get_relative_position();
            let moon_pos = earth_pos + celestial_bodies["Moon"].get_relative_position();
            camera.position = moon_pos + Vec3::new(0.0, 0.3, 0.6);
            camera.look_at(moon_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num4) {
            let mars_pos = celestial_bodies["Mars"].get_relative_position();
            camera.position = mars_pos + Vec3::new(0.0, 0.3, 0.6);
            camera.look_at(mars_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num5) {
            let mercury_pos = celestial_bodies["Mercury"].get_relative_position();
            camera.position = mercury_pos + Vec3::new(0.0, 1.0, 2.5);
            camera.look_at(mercury_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num6) {
            let venus_pos = celestial_bodies["Venus"].get_relative_position();
            camera.position = venus_pos + Vec3::new(0.0, 1.5, 3.5);
            camera.look_at(venus_pos);
        }

        let fwd = camera.forward();
        let right = fwd.cross(camera.up).normalize();
        let mut vel = Vec3::ZERO;
        if input_handler.keys_pressed.contains(&glfw::Key::W) {
            vel += fwd;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::S) {
            vel -= fwd;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::A) {
            vel -= right;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::D) {
            vel += right;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Space) {
            vel += camera.up;
        }
        if input_handler.keys_pressed.contains(&glfw::Key::LeftShift) {
            vel -= camera.up;
        }
        if vel.length_squared() > 0.0 {
            camera.position += vel.normalize() * MOVE_SPEED * delta;
        }

        let view = camera.view_matrix();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let mut ts = TransformStack::new();
        let mut renderables = Vec::new();

        // SUN (at origin)
        let sun = &celestial_bodies["Sun"];
        {
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
        }

        // PLANETS orbiting the Sun
        for planet_name in ["Mercury", "Venus", "Earth", "Mars"] {
            let planet = &celestial_bodies[planet_name];
            
            // Push planet's orbital transform around Sun
            ts.push(Mat4::from_translation(planet.get_relative_position()));
            
            // Planet's own rotation and scale
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

            // MOON and SPACECRAFT (only for Earth)
            if planet_name == "Earth" {
                let moon = &celestial_bodies["Moon"];
                
                // Push Moon's orbital transform around Earth
                ts.push(Mat4::from_translation(moon.get_relative_position()));
                
                // Moon's own rotation and scale
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

                // SPACECRAFT relative to Moon
                ts.push(Mat4::from_quat(spacecraft_rot));
                ts.push(Mat4::from_translation(Vec3::new(0.0, 0.0, spacecraft_dist)));
                let spacecraft_model = ts.current()
                    * Mat4::from_scale(Vec3::splat(SPACECRAFT_SIZE));

                renderables.push(Renderable {
                    geometry: Rc::clone(&spacecraft),
                    model_matrix: spacecraft_model,
                    emit_mode: 0,
                    use_texture: true,
                    texture_id: spacecraft_texture,
                });
                // Pop spacecraft transforms
                ts.pop(); // distance
                ts.pop(); // rotation
                // Pop Moon position
                ts.pop();
            }
            
            // Pop planet position
            ts.pop();
        }

        // === SHADOW PASS ===
        shadow_renderer.render_depth_pass(&renderables);
        unsafe {
            gl::UseProgram(shader_program);
        }
        unsafe {
            gl::Viewport(0, 0, win_width, win_height);
        }

        // === FORWARD PASS ===
        unsafe {
            if view_loc >= 0 {
                gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.to_cols_array().as_ptr());
            }
            if proj_loc >= 0 {
                gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());
            }

            let light_world = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let light_view = view * light_world;
            if light_pos_loc >= 0 {
                gl::Uniform4f(light_pos_loc, light_view.x, light_view.y, light_view.z, 1.0);
            }
            if light_pos_world_loc >= 0 {
                gl::Uniform3f(light_pos_world_loc, 0.0, 0.0, 0.0);
            }

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, shadow_renderer.depth_cubemap);
            if shadow_map_loc >= 0 {
                gl::Uniform1i(shadow_map_loc, 1);
            }
            if shadow_far_loc >= 0 {
                gl::Uniform1f(shadow_far_loc, shadow_renderer.shadow_far);
            }
            gl::Viewport(0, 0, win_width, win_height);
        }

        skybox.draw(&view, &projection);
        for r in &renderables {
            let normal_matrix = Mat3::from_mat4(r.model_matrix).inverse().transpose();
            unsafe {
                if model_loc >= 0 {
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, r.model_matrix.to_cols_array().as_ptr());
                }
                if normal_matrix_loc >= 0 {
                    gl::UniformMatrix3fv(normal_matrix_loc, 1, gl::FALSE, normal_matrix.to_cols_array().as_ptr());
                }
                if emit_mode_loc >= 0 {
                    gl::Uniform1ui(emit_mode_loc, r.emit_mode);
                }
                if use_texture_loc >= 0 {
                    gl::Uniform1i(use_texture_loc, if r.use_texture { 1 } else { 0 });
                }
                if r.use_texture {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, r.texture_id);
                    if base_texture_loc >= 0 {
                        gl::Uniform1i(base_texture_loc, 0);
                    }
                }
                r.geometry.draw();
            }
        }

        window.swap_buffers();
    }

    // Cleanup
    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteTextures(1, &earth_texture);
        gl::DeleteTextures(1, &sun_texture);
        gl::DeleteTextures(1, &moon_texture);
        gl::DeleteTextures(1, &mercury_texture);
        gl::DeleteTextures(1, &venus_texture);
        gl::DeleteTextures(1, &mars_texture);
        gl::DeleteTextures(1, &skybox_texture);
    }
    shadow_renderer.destroy();
    Ok(())
}