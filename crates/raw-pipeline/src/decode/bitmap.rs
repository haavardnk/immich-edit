use super::format_hint;
use crate::PipelineError;
use crate::frame::RawFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InputFormat {
    Jpeg,
    Png,
    Tiff,
    Webp,
    Heif,
    Jxl,
    Gif,
    Bmp,
}

pub(super) fn sniff_format(data: &[u8]) -> Option<InputFormat> {
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some(InputFormat::Jpeg);
    }
    if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        return Some(InputFormat::Png);
    }
    if data.len() >= 4 && (&data[0..4] == b"II*\0" || &data[0..4] == b"MM\0*") {
        return Some(InputFormat::Tiff);
    }
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some(InputFormat::Webp);
    }
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        let brand = &data[8..12];
        if matches!(
            brand,
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1" | b"avif" | b"avis"
        ) {
            return Some(InputFormat::Heif);
        }
    }
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0x0A {
        return Some(InputFormat::Jxl);
    }
    if data.len() >= 12 && &data[0..12] == b"\0\0\0\x0CJXL \r\n\x87\n" {
        return Some(InputFormat::Jxl);
    }
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return Some(InputFormat::Gif);
    }
    if data.len() >= 2 && &data[0..2] == b"BM" {
        return Some(InputFormat::Bmp);
    }
    None
}

pub(super) fn decode_image(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    match sniff_format(data) {
        Some(InputFormat::Jpeg) => decode_jpeg(data, exif),
        Some(InputFormat::Png) => decode_png(data, exif),
        Some(InputFormat::Tiff) => decode_tiff(data, exif),
        Some(InputFormat::Webp) => decode_webp(data, exif),
        Some(InputFormat::Heif) => decode_heif(data, exif),
        Some(InputFormat::Jxl) => decode_jxl(data, exif),
        Some(InputFormat::Gif) => decode_via_image_crate(data, exif),
        Some(InputFormat::Bmp) => decode_via_image_crate(data, exif),
        None => Err(PipelineError::Unsupported(format!(
            "unknown image format ({})",
            format_hint(data)
        ))),
    }
}

fn frame_from_rgb8(
    rgb: Vec<u8>,
    width: usize,
    height: usize,
    exif: Option<little_exif::metadata::Metadata>,
) -> RawFrame {
    let linear: Vec<f32> = rgb
        .iter()
        .map(|&v| srgb_to_linear(v as f32 / 255.0))
        .collect();
    let orientation = exif
        .as_ref()
        .and_then(crate::exif::orientation)
        .unwrap_or((false, false, false));
    RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 8,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        xyz_to_cam: identity_xyz_to_cam(),
        color_matrices: Vec::new(),
        data: linear,
        cpp: 3,
        orientation,
        is_raw: false,
        capture_sigma: None,
        model: String::new(),
        exif,
    }
}

fn frame_from_rgb16(
    rgb: Vec<u16>,
    width: usize,
    height: usize,
    exif: Option<little_exif::metadata::Metadata>,
) -> RawFrame {
    let linear: Vec<f32> = rgb
        .iter()
        .map(|&v| srgb_to_linear(v as f32 / 65535.0))
        .collect();
    let orientation = exif
        .as_ref()
        .and_then(crate::exif::orientation)
        .unwrap_or((false, false, false));
    RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 16,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        xyz_to_cam: identity_xyz_to_cam(),
        color_matrices: Vec::new(),
        data: linear,
        cpp: 3,
        orientation,
        is_raw: false,
        capture_sigma: None,
        model: String::new(),
        exif,
    }
}

fn identity_xyz_to_cam() -> [[f32; 3]; 4] {
    [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    ]
}

fn decode_jpeg(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let mut decompressor = turbojpeg::Decompressor::new()
        .map_err(|e| PipelineError::Decode(format!("jpeg init: {e}")))?;
    let header = decompressor
        .read_header(data)
        .map_err(|e| PipelineError::Decode(format!("jpeg header: {e}")))?;
    let width = header.width;
    let height = header.height;
    let mut rgb_buf = vec![0u8; width * height * 3];
    let image = turbojpeg::Image {
        pixels: rgb_buf.as_mut_slice(),
        width,
        pitch: width * 3,
        height,
        format: turbojpeg::PixelFormat::RGB,
    };
    decompressor
        .decompress(data, image)
        .map_err(|e| PipelineError::Decode(format!("jpeg decompress: {e}")))?;
    Ok(frame_from_rgb8(rgb_buf, width, height, exif))
}

fn decode_png(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let decoder = png::Decoder::new(std::io::Cursor::new(data));
    let mut reader = decoder
        .read_info()
        .map_err(|e| PipelineError::Decode(format!("png: {e}")))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| PipelineError::Decode(format!("png: {e}")))?;
    let width = info.width as usize;
    let height = info.height as usize;
    let bytes = &buf[..frame.buffer_size()];
    match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            Ok(frame_from_rgb8(bytes.to_vec(), width, height, exif))
        }
        (png::ColorType::Rgba, png::BitDepth::Eight) => {
            let rgb = rgba8_to_rgb8(bytes);
            Ok(frame_from_rgb8(rgb, width, height, exif))
        }
        (png::ColorType::Rgb, png::BitDepth::Sixteen) => {
            let rgb = be_bytes_to_u16(bytes);
            Ok(frame_from_rgb16(rgb, width, height, exif))
        }
        (png::ColorType::Rgba, png::BitDepth::Sixteen) => {
            let rgba = be_bytes_to_u16(bytes);
            let rgb = rgba16_to_rgb16(&rgba);
            Ok(frame_from_rgb16(rgb, width, height, exif))
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            let rgb = gray8_to_rgb8(bytes);
            Ok(frame_from_rgb8(rgb, width, height, exif))
        }
        (png::ColorType::Grayscale, png::BitDepth::Sixteen) => {
            let g = be_bytes_to_u16(bytes);
            let rgb: Vec<u16> = g.iter().flat_map(|&v| [v, v, v]).collect();
            Ok(frame_from_rgb16(rgb, width, height, exif))
        }
        (ct, bd) => Err(PipelineError::Unsupported(format!(
            "png: unsupported color {ct:?} depth {bd:?}"
        ))),
    }
}

fn decode_tiff(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    use tiff::decoder::DecodingResult;
    let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(data))
        .map_err(|e| PipelineError::Decode(format!("tiff: {e}")))?;
    let (w, h) = decoder
        .dimensions()
        .map_err(|e| PipelineError::Decode(format!("tiff dims: {e}")))?;
    let width = w as usize;
    let height = h as usize;
    let colortype = decoder
        .colortype()
        .map_err(|e| PipelineError::Decode(format!("tiff colortype: {e}")))?;
    let result = decoder
        .read_image()
        .map_err(|e| PipelineError::Decode(format!("tiff read: {e}")))?;
    match (colortype, result) {
        (tiff::ColorType::RGB(8), DecodingResult::U8(v)) => {
            Ok(frame_from_rgb8(v, width, height, exif))
        }
        (tiff::ColorType::RGBA(8), DecodingResult::U8(v)) => {
            Ok(frame_from_rgb8(rgba8_to_rgb8(&v), width, height, exif))
        }
        (tiff::ColorType::RGB(16), DecodingResult::U16(v)) => {
            Ok(frame_from_rgb16(v, width, height, exif))
        }
        (tiff::ColorType::RGBA(16), DecodingResult::U16(v)) => {
            Ok(frame_from_rgb16(rgba16_to_rgb16(&v), width, height, exif))
        }
        (tiff::ColorType::Gray(8), DecodingResult::U8(v)) => {
            Ok(frame_from_rgb8(gray8_to_rgb8(&v), width, height, exif))
        }
        (tiff::ColorType::Gray(16), DecodingResult::U16(v)) => {
            let rgb: Vec<u16> = v.iter().flat_map(|&g| [g, g, g]).collect();
            Ok(frame_from_rgb16(rgb, width, height, exif))
        }
        (ct, _) => Err(PipelineError::Unsupported(format!(
            "tiff: unsupported colortype {ct:?}"
        ))),
    }
}

fn decode_webp(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let decoder = webp::Decoder::new(data);
    let webp_image = decoder
        .decode()
        .ok_or_else(|| PipelineError::Decode("webp decode failed".into()))?;
    let width = webp_image.width() as usize;
    let height = webp_image.height() as usize;
    let bytes = webp_image.to_vec();
    let rgb = if webp_image.is_alpha() {
        rgba8_to_rgb8(&bytes)
    } else {
        bytes
    };
    Ok(frame_from_rgb8(rgb, width, height, exif))
}

fn decode_heif(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    use libheif_rs::{ColorSpace, HeifContext, HeifErrorCode, LibHeif, RgbChroma};
    let lib_heif = LibHeif::new();
    let ctx = HeifContext::read_from_bytes(data)
        .map_err(|e| PipelineError::Decode(format!("heif: {e}")))?;
    let handle = ctx
        .primary_image_handle()
        .map_err(|e| PipelineError::Decode(format!("heif handle: {e}")))?;
    let image = lib_heif
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .map_err(|e| {
            let hint = match e.code {
                HeifErrorCode::DecoderPluginError
                | HeifErrorCode::PluginLoadingError
                | HeifErrorCode::UnsupportedFileType => {
                    "; codec plugin missing (install libheif-plugin-libde265 for HEIC, libheif-plugin-dav1d for AVIF)"
                }
                _ => "",
            };
            PipelineError::Decode(format!("heif decode: {e}{hint}"))
        })?;
    let width = image.width() as usize;
    let height = image.height() as usize;
    let planes = image.planes();
    let plane = planes
        .interleaved
        .ok_or_else(|| PipelineError::Decode("heif: no interleaved plane".into()))?;
    let stride = plane.stride;
    let row_bytes = width * 3;
    let mut rgb = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        let off = y * stride;
        rgb.extend_from_slice(&plane.data[off..off + row_bytes]);
    }
    let mut frame = frame_from_rgb8(rgb, width, height, exif);
    frame.orientation = (false, false, false);
    Ok(frame)
}

fn decode_jxl(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    use jpegxl_rs::decode::PixelFormat;
    let decoder = jpegxl_rs::decoder_builder()
        .pixel_format(PixelFormat {
            num_channels: 3,
            ..Default::default()
        })
        .build()
        .map_err(|e| PipelineError::Decode(format!("jxl init: {e}")))?;
    let (meta, pixels) = decoder
        .decode_with::<u8>(data)
        .map_err(|e| PipelineError::Decode(format!("jxl decode: {e}")))?;
    let width = meta.width as usize;
    let height = meta.height as usize;
    Ok(frame_from_rgb8(pixels, width, height, exif))
}

fn decode_via_image_crate(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let img =
        image::load_from_memory(data).map_err(|e| PipelineError::Decode(format!("image: {e}")))?;
    let rgb = img.to_rgb8();
    let width = rgb.width() as usize;
    let height = rgb.height() as usize;
    Ok(frame_from_rgb8(rgb.into_raw(), width, height, exif))
}

fn rgba8_to_rgb8(rgba: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    rgb
}

fn rgba16_to_rgb16(rgba: &[u16]) -> Vec<u16> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    rgb
}

fn gray8_to_rgb8(g: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(g.len() * 3);
    for &v in g {
        rgb.extend_from_slice(&[v, v, v]);
    }
    rgb
}

fn be_bytes_to_u16(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect()
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
