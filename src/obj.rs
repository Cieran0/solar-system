use std::{collections::HashMap, error::Error, fs::read_to_string};
use gl::types::{GLuint, GLsizei};
use crate::shape::Shape;
use glam::Mat4;

struct ObjData {
    pub positions: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(PartialEq)]
enum ParserState {
    NewLine,
    ReadingVertex,
    ReadingNormal,
    ReadingTexture,
    ReadingFace,
}

fn parse_face_vertex(field: &str) -> Result<(Option<u32>, Option<u32>, Option<u32>), Box<dyn Error>> {
    let parts: Vec<&str> = field.split('/').collect();
    let pos_idx = if parts[0].is_empty() { None } else { Some(parts[0].parse()?) };
    let tex_idx = if parts.get(1).map_or(true, |s| s.is_empty()) { None } else { Some(parts[1].parse()?) };
    let norm_idx = if parts.get(2).map_or(true, |s| s.is_empty()) { None } else { Some(parts[2].parse()?) };
    Ok((pos_idx, tex_idx, norm_idx))
}

fn load_obj_from_file(path: &str) -> Result<ObjData, Box<dyn Error>> {
    let data = read_to_string(path)?;
    let lines = data.lines();

    let mut buff: Vec<f32> = vec![];

    let mut raw_positions = Vec::new();
    let mut raw_tex_coords = Vec::new();
    let mut raw_normals = Vec::new();

    let mut positions = Vec::new();
    let mut tex_coords = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let mut vertex_map: HashMap<(usize, usize, usize), u32> = HashMap::new();
    let mut next_index = 0u32;

    let mut parser_state = ParserState::NewLine;

    for line in lines {
        match parser_state {
            ParserState::NewLine => {},
            ParserState::ReadingVertex => {
                raw_positions.push([buff[0], buff[1], buff[2]]);
                buff.clear();
            },
            ParserState::ReadingNormal => {
                raw_normals.push([buff[0], buff[1], buff[2]]);
                buff.clear();
            },
            ParserState::ReadingTexture => {
                raw_tex_coords.push([buff[0], buff[1]]);
                buff.clear();
            },
            ParserState::ReadingFace => {
                // Handled after state update
            },
        }

        parser_state = ParserState::NewLine;
        let split_line: Vec<&str> = line.split_ascii_whitespace().collect();

        if split_line.is_empty() {
            continue;
        }

        let first = split_line[0];
        if first.starts_with("#") {
            continue;
        }

        parser_state = match first {
            "v" => ParserState::ReadingVertex,
            "vn" => ParserState::ReadingNormal,
            "vt" => ParserState::ReadingTexture,
            "f" => ParserState::ReadingFace,
            _ => continue,
        };

        if parser_state == ParserState::ReadingFace {
            let mut face_vertices = Vec::new();
            for field in &split_line[1..] {
                let (pos, tex, norm) = parse_face_vertex(field)?;
                let pos = pos.map(|i| (i - 1) as usize);
                let tex = tex.map(|i| (i - 1) as usize);
                let norm = norm.map(|i| (i - 1) as usize);
                face_vertices.push((pos, tex, norm));
            }

            if face_vertices.len() < 3 {
                return Err("Face with fewer than 3 vertices".into());
            }

            for i in 1..face_vertices.len() - 1 {
                let verts = [
                    face_vertices[0],
                    face_vertices[i],
                    face_vertices[i + 1],
                ];

                for &(p, t, n) in &verts {
                    let p = p.ok_or("Missing position index in face")?;
                    let t = t.unwrap_or(0);
                    let n = n.unwrap_or(0);

                    let key = (p, t, n);
                    let final_index = if let Some(&idx) = vertex_map.get(&key) {
                        idx
                    } else {
                        positions.push(raw_positions[p]);
                        tex_coords.push(raw_tex_coords.get(t).copied().unwrap_or([0.0, 0.0]));
                        normals.push(raw_normals.get(n).copied().unwrap_or([0.0, 0.0, 1.0]));
                        vertex_map.insert(key, next_index);
                        let idx = next_index;
                        next_index += 1;
                        idx
                    };
                    indices.push(final_index);
                }
            }

            continue;
        }

        // Parse numeric components for v, vt, vn
        for seg in &split_line[1..] {
            if matches!(parser_state, ParserState::ReadingVertex | ParserState::ReadingNormal | ParserState::ReadingTexture) {
                let value: f32 = seg.parse()?;
                buff.push(value);
            }
        }
    }

    // Finalize last token if needed (edge case)
    match parser_state {
        ParserState::ReadingVertex if buff.len() == 3 => {
            raw_positions.push([buff[0], buff[1], buff[2]]);
        }
        ParserState::ReadingNormal if buff.len() == 3 => {
            raw_normals.push([buff[0], buff[1], buff[2]]);
        }
        ParserState::ReadingTexture if buff.len() == 2 => {
            raw_tex_coords.push([buff[0], buff[1]]);
        }
        _ => {}
    }

    Ok(ObjData {
        positions,
        tex_coords,
        normals,
        indices,
    })
}

pub struct ObjModel {
    vao: GLuint,
    vbo: [GLuint; 5], // pos, colour, normal, texcoord, indices
    index_count: GLsizei,
}

impl ObjModel {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = load_obj_from_file(path)?;

        let positions = data.positions;
        let tex_coords = data.tex_coords;
        let normals = data.normals;
        let indices = data.indices;

        // Default white vertex colours (since OBJ has no per-vertex colour)
        let colours: Vec<[f32; 4]> = vec![[1.0, 1.0, 1.0, 1.0]; positions.len()];

        unsafe {
            let mut vao = 0;
            let mut vbo = [0u32; 5];
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(5, vbo.as_mut_ptr());

            gl::BindVertexArray(vao);

            // Position (location 0)
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (positions.len() * std::mem::size_of::<[f32; 3]>()) as isize,
                positions.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Colour (location 1) – white fallback
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (colours.len() * std::mem::size_of::<[f32; 4]>()) as isize,
                colours.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(1);

            // Normal (location 2)
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[2]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (normals.len() * std::mem::size_of::<[f32; 3]>()) as isize,
                normals.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(2);

            // TexCoord (location 3)
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[3]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (tex_coords.len() * std::mem::size_of::<[f32; 2]>()) as isize,
                tex_coords.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(3);

            // Index buffer (ELEMENT_ARRAY_BUFFER)
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo[4]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);

            Ok(Self {
                vao,
                vbo,
                index_count: indices.len() as GLsizei,
            })
        }
    }
}

impl Shape for ObjModel {
    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}

impl Drop for ObjModel {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(5, self.vbo.as_ptr());
        }
    }
}