// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::error::Error;

use crate::qoi::{QoiImage};

// Creates a 2D OpenGL texture from a QOI image and returns its texture ID.
pub fn create_texture_from_image(img: &QoiImage) -> Result<u32, Box<dyn Error>> {
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
            img.desc.width as i32,
            img.desc.height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.data.as_ptr() as *const _,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    Ok(texture_id)
}

// Creates an OpenGL cubemap texture from six QOI images (one for each face) and returns its texture ID.
pub fn create_cubemap_from_images(images: [&QoiImage; 6]) -> Result<u32, Box<dyn Error>> {
    let mut cubemap_id = 0;
    unsafe {
        gl::GenTextures(1, &mut cubemap_id);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_id);
        
        const CUBE_MAP_FACES: [u32; 6] = [
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        ];
        
        for i in 0..6 {
            let raw = &images[i];
            gl::TexImage2D(
                CUBE_MAP_FACES[i],
                0,
                gl::RGBA8 as i32,
                raw.desc.width as i32,
                raw.desc.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                raw.data.as_ptr() as *const _,
            );
        }
        
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