mod window;
mod camera;
mod shaders;
mod transform_stack;
mod shape;
mod obj;
mod shadow;
mod skybox;
mod qoi;
mod texture;
mod uniforms;
mod solar_system;

use std::{
    collections::HashMap,
    error::Error,
    io::{self, Read, Write},
};

use crate::solar_system::SolarSystem;
use crate::window::Window;

fn print_controls() {
    println!("=== Controls ===");
    println!("Camera movement: W/A/S/D + Space (up) / LeftShift (down)");
    println!("Camera views (Hold): 1 = Sun, 2 = Earth, 3 = Moon, 4 = Mars, 5 = Mercury, 6 = Venus");
    println!("Simulation speed: '=' = faster, '-' = slower");
    println!("Spacecraft rotation: Arrow keys");
    println!("Spacecraft distance from Moon: PageUp / PageDown");
    println!("================");
}

fn load_images(root: &str) -> Result<HashMap<String, crate::qoi::QoiImage>, Box<dyn Error>> {
    use walkdir::WalkDir;
    let paths: Vec<String> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("qoi")))
        .filter_map(|p| p.to_str().map(|s| s.to_owned()))
        .collect();

    if paths.is_empty() {
        return Ok(HashMap::new());
    }

    let mut buffers = Vec::with_capacity(paths.len());
    for path in &paths {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        buffers.push((path.clone(), buffer));
    }

    let (tx, rx) = std::sync::mpsc::channel();
    for (path, buf) in buffers {
        let tx = tx.clone();
        std::thread::spawn(move || {
            match crate::qoi::decode(&buf) {
                Ok(image) => drop(tx.send(Ok((path, image)))),
                Err(e) => drop(tx.send(Err(format!("Failed to decode {}: {}", path, e)))),
            }
        });
    }

    drop(tx);
    let mut images = HashMap::with_capacity(paths.len());
    for _ in 0..paths.len() {
        match rx.recv()? {
            Ok((path, image)) => {
                images.insert(path, image);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(images)
}

fn main() -> Result<(), Box<dyn Error>> {
    print_controls();
    print!("Loading images...");
    io::stdout().flush()?;

    let images = load_images("textures")?;
    println!("Done!");

    let width = 1280u32;
    let height = 720u32;
    let window = Window::new(width, height, "Solar System")?;

    let mut solar_system = SolarSystem::new(window, images)?;

    let mut last_time = std::time::Instant::now();

    let mut fps_timer = std::time::Instant::now();
    let mut frame_count: u32 = 0;

    while !solar_system.should_close() {
        let now = std::time::Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;

        // FPS counting
        frame_count += 1;
        if fps_timer.elapsed().as_secs_f32() >= 1.0 {
            println!("FPS: {}", frame_count);
            frame_count = 0;
            fps_timer = std::time::Instant::now();
        }

        solar_system.handle_events();
        solar_system.update(delta);
        solar_system.draw();
        solar_system.swap_buffers();
    }

    solar_system.cleanup();
    Ok(())
}
