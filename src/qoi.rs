use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Copy, Clone, Debug)]
pub struct QoiDesc {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
}

#[derive(Copy, Clone, Debug)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const QOI_PADDING: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const QOI_HEADER_SIZE: usize = 14;
const QOI_PIXELS_MAX: u32 = 400_000_000;

const QOI_OP_INDEX: u8 = 0x00;
const QOI_OP_DIFF: u8 = 0x40;
const QOI_OP_LUMA: u8 = 0x80;
const QOI_OP_RUN: u8 = 0xC0;
const QOI_OP_RGB: u8 = 0xFE;
const QOI_OP_RGBA: u8 = 0xFF;

const QOI_MASK_2: u8 = 0xC0;

const QOI_MAGIC: u32 = ('q' as u32) << 24 | ('o' as u32) << 16 | ('i' as u32) << 8 | ('f' as u32);


fn colour_hash(px: Rgba) -> usize {
    px.r as usize * 3 +
    px.g as usize * 5 +
    px.b as usize * 7 +
    px.a as usize * 11
}

fn read_u32_be(bytes: &[u8], p: &mut usize) -> Result<u32, String> {
    if *p + 4 > bytes.len() {
        return Err("Unexpected EOF reading header".into());
    }
    let v = ((bytes[*p] as u32) << 24)
          | ((bytes[*p + 1] as u32) << 16)
          | ((bytes[*p + 2] as u32) << 8)
          | (bytes[*p + 3] as u32);
    *p += 4;
    Ok(v)
}

fn parse_header(bytes: &[u8], desc: &mut QoiDesc) -> Result<usize, String> {
    if bytes.len() < QOI_HEADER_SIZE {
        return Err("Too small to be a QOI file".into());
    }

    let mut p = 0;
    let magic = read_u32_be(bytes, &mut p)?;
    desc.width = read_u32_be(bytes, &mut p)?;
    desc.height = read_u32_be(bytes, &mut p)?;
    desc.channels = *bytes.get(p).ok_or("Missing channels")?;
    p += 1;
    desc.colorspace = *bytes.get(p).ok_or("Missing colorspace")?;
    p += 1;

    if magic != QOI_MAGIC {
        return Err("Invalid QOI magic".into());
    }
    if desc.width == 0 || desc.height == 0 {
        return Err("Invalid QOI dimensions".into());
    }
    if !matches!(desc.channels, 3 | 4) {
        return Err("Invalid channel count".into());
    }
    if desc.colorspace > 1 {
        return Err("Invalid colorspace".into());
    }
    if desc.height >= QOI_PIXELS_MAX / desc.width {
        return Err("Image too large".into());
    }

    Ok(p)
}

fn write_pixel(out: &mut [u8], pos: usize, px: Rgba, channels: usize) {
    out[pos] = px.r;
    out[pos + 1] = px.g;
    out[pos + 2] = px.b;
    if channels == 4 {
        out[pos + 3] = px.a;
    }
}

fn process_op(
    b1: u8,
    bytes: &[u8],
    p: &mut usize,
    px: &mut Rgba,
    index: &mut [Rgba; 64],
    run: &mut usize,
) -> Result<(), String> {
    match b1 {
        QOI_OP_RGB => {
            px.r = *bytes.get(*p).ok_or("EOF in RGB")?;
            px.g = *bytes.get(*p + 1).ok_or("EOF in RGB")?;
            px.b = *bytes.get(*p + 2).ok_or("EOF in RGB")?;
            *p += 3;
        }
        QOI_OP_RGBA => {
            px.r = *bytes.get(*p).ok_or("EOF in RGBA")?;
            px.g = *bytes.get(*p + 1).ok_or("EOF in RGBA")?;
            px.b = *bytes.get(*p + 2).ok_or("EOF in RGBA")?;
            px.a = *bytes.get(*p + 3).ok_or("EOF in RGBA")?;
            *p += 4;
        }
        _ if (b1 & QOI_MASK_2) == QOI_OP_INDEX => {
            *px = index[b1 as usize];
        }
        _ if (b1 & QOI_MASK_2) == QOI_OP_DIFF => {
            px.r = px.r.wrapping_add(((b1 >> 4) & 0x03).wrapping_sub(2));
            px.g = px.g.wrapping_add(((b1 >> 2) & 0x03).wrapping_sub(2));
            px.b = px.b.wrapping_add((b1 & 0x03).wrapping_sub(2));
        }
        _ if (b1 & QOI_MASK_2) == QOI_OP_LUMA => {
            let b2 = *bytes.get(*p).ok_or("EOF in LUMA")?;
            *p += 1;

            let vg = (b1 & 0x3f) as i8 - 32;

            px.r = px.r.wrapping_add((vg - 8 + ((b2 >> 4) & 0x0f) as i8) as u8);
            px.g = px.g.wrapping_add(vg as u8);
            px.b = px.b.wrapping_add((vg - 8 + (b2 & 0x0f) as i8) as u8);
        }
        _ if (b1 & QOI_MASK_2) == QOI_OP_RUN => {
            *run = (b1 & 0x3f) as usize;
        }
        _ => {}
    };

    index[colour_hash(*px) & 63] = *px;

    Ok(())
}


fn decode_chunks(bytes: &[u8], mut p: usize, desc: &QoiDesc, channels: usize) -> Result<Vec<u8>, String> {

    let total_px = desc.width as usize * desc.height as usize * channels;
    let mut out = vec![0u8; total_px];

    let chunks_end = bytes.len() - QOI_PADDING.len();
    let mut index = [Rgba { r: 0, g: 0, b: 0, a: 0 }; 64];
    let mut px = Rgba { r: 0, g: 0, b: 0, a: 255 };

    let mut run = 0usize;
    let mut pos = 0;

    while pos < total_px {
        if run > 0 {
            run -= 1;
        } else if p < chunks_end {
            let b1 = bytes[p];
            p += 1;
            process_op(b1, bytes, &mut p, &mut px, &mut index, &mut run)?;
        }

        write_pixel(&mut out, pos, px, channels);
        pos += channels;
    }

    Ok(out)
}


pub fn decode(bytes: &[u8], req_channels: usize, desc: &mut QoiDesc) -> Result<Vec<u8>, String> {
    let p = parse_header(bytes, desc)?;

    let channels = if req_channels == 0 {
        desc.channels as usize
    } else {
        req_channels
    };

    decode_chunks(bytes, p, desc, channels)
}

#[derive(Debug)]
pub struct RawImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn read_file(filename: &str, desc: &mut QoiDesc, channels: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(filename)
        .map_err(|e| format!("Failed to open {}: {}", filename, e))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read {}: {}", filename, e))?;

    decode(&data, channels, desc)
}

pub fn read_as_raw(path: &str) -> Result<RawImage, Box<dyn Error>> {
    let mut desc = QoiDesc { width: 0, height: 0, channels: 0, colorspace: 0 };
    let data = read_file(path, &mut desc, 4)?;
    Ok(RawImage {
        data,
        width: desc.width,
        height: desc.height,
    })
}