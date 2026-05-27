use crate::PipelineError;
use crate::frame::RawFrame;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::rawimage::RawPhotometricInterpretation;

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_fast(raw_image, exif),
        Err(e) => raw_fallback_or_unsupported(e, data, exif),
    }
}

pub fn decode_quality(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_quality(raw_image, exif),
        Err(e) => raw_fallback_or_unsupported(e, data, exif),
    }
}

fn raw_fallback_or_unsupported(
    err: impl std::fmt::Display,
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let msg = format!("{err}");
    if msg.contains("No decoder found") {
        decode_image(data, exif)
    } else {
        Err(PipelineError::Unsupported(format!(
            "RAW format not supported by rawler ({}): {msg}",
            format_hint(data)
        )))
    }
}

fn format_hint(data: &[u8]) -> String {
    match sniff_format(data) {
        Some(f) => format!("{f:?}"),
        None => {
            let head: Vec<String> = data.iter().take(4).map(|b| format!("{b:02X}")).collect();
            if head.is_empty() {
                "empty".into()
            } else {
                format!("magic {}", head.join(" "))
            }
        }
    }
}

type ExtractedMeta = (
    [f32; 4],
    [[f32; 3]; 4],
    Vec<(f32, [[f32; 3]; 4])>,
    crate::frame::OrientFlips,
);

fn extract_common(
    raw_image: &mut rawler::RawImage,
    exif: &Option<little_exif::metadata::Metadata>,
) -> ExtractedMeta {
    let wb_coeffs = raw_image.wb_coeffs;
    let color_matrices = extract_color_matrices(raw_image);
    populate_xyz_to_cam_from_color_matrix(raw_image);
    let xyz_to_cam = raw_image.xyz_to_cam;
    let orientation = exif
        .as_ref()
        .and_then(crate::exif::orientation)
        .unwrap_or_else(|| raw_image.orientation.to_flips());
    (wb_coeffs, xyz_to_cam, color_matrices, orientation)
}

fn illuminant_to_cct(illu: &rawler::imgop::xyz::Illuminant) -> f32 {
    use rawler::imgop::xyz::Illuminant::*;
    match illu {
        A | Tungsten => 2856.0,
        B => 4874.0,
        C => 6774.0,
        D50 => 5003.0,
        D55 => 5503.0,
        D65 => 6504.0,
        D75 => 7504.0,
        Daylight | FineWeather | Flash => 5500.0,
        Fluorescent => 4150.0,
        CloudyWeather => 6500.0,
        Shade => 7500.0,
        DaylightFluorescent => 6430.0,
        DaylightWhiteFluorescent => 5000.0,
        CoolWhiteFluorescent => 4150.0,
        WhiteFluorescent => 3450.0,
        IsoStudioTungsten => 3200.0,
        Unknown => 6504.0,
    }
}

fn extract_color_matrices(raw_image: &rawler::RawImage) -> Vec<(f32, [[f32; 3]; 4])> {
    let mut result = Vec::new();
    for (illu, matrix) in &raw_image.color_matrix {
        if matrix.len() % 3 != 0 {
            continue;
        }
        let components = (matrix.len() / 3).min(4);
        let mut xyz_to_cam = [[0.0f32; 3]; 4];
        for i in 0..components {
            for j in 0..3 {
                xyz_to_cam[i][j] = matrix[i * 3 + j];
            }
        }
        let cct = illuminant_to_cct(illu);
        result.push((cct, xyz_to_cam));
    }
    result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    result
}

fn populate_xyz_to_cam_from_color_matrix(raw_image: &mut rawler::RawImage) {
    if raw_image
        .xyz_to_cam
        .iter()
        .any(|row| row.iter().any(|v| *v != 0.0))
    {
        return;
    }
    let matrix = raw_image
        .color_matrix
        .iter()
        .find(|(illu, _)| **illu == rawler::imgop::xyz::Illuminant::D65)
        .map(|(_, m)| m)
        .or_else(|| raw_image.color_matrix.values().next());
    let Some(matrix) = matrix else {
        return;
    };
    if matrix.len() % 3 != 0 {
        return;
    }
    let components = (matrix.len() / 3).min(4);
    let mut xyz_to_cam = [[0.0f32; 3]; 4];
    for i in 0..components {
        for j in 0..3 {
            xyz_to_cam[i][j] = matrix[i * 3 + j];
        }
    }
    raw_image.xyz_to_cam = xyz_to_cam;
}

fn decode_raw_fast(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp != 1 {
        return decode_raw_quality(raw_image, exif);
    }
    let cfa_name = match &raw_image.photometric {
        RawPhotometricInterpretation::Cfa(config) if config.cfa.is_rgb() => config.cfa.name.clone(),
        _ => return decode_raw_quality(raw_image, exif),
    };

    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let develop = RawDevelop {
        steps: vec![ProcessingStep::Rescale],
    };
    let intermediate = develop
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let pixels = match intermediate {
        Intermediate::Monochrome(p) => p,
        _ => return decode_raw_quality(raw_image, exif),
    };

    let (data, width, height, cfa_pattern) = if let Some(area) = raw_image.active_area {
        let cropped = pixels.crop(area);
        let shifted = shift_cfa(&cfa_name, area.p.x, area.p.y);
        let w = cropped.width;
        let h = cropped.height;
        (cropped.into_inner(), w, h, shifted)
    } else {
        let w = pixels.width;
        let h = pixels.height;
        (pixels.into_inner(), w, h, cfa_name)
    };

    Ok(RawFrame {
        width,
        height,
        cfa_pattern,
        bps: 16,
        wb_coeffs,
        xyz_to_cam,
        color_matrices,
        data,
        cpp: 1,
        orientation,
        is_raw: true,
        exif,
    })
}

fn decode_raw_quality(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let develop = RawDevelop {
        steps: vec![
            ProcessingStep::Rescale,
            ProcessingStep::Demosaic,
            ProcessingStep::CropActiveArea,
        ],
    };
    let intermediate = develop
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let (data, width, height) = match intermediate {
        Intermediate::ThreeColor(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels.into_inner().into_iter().flatten().collect();
            (flat, w, h)
        }
        Intermediate::FourColor(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels
                .into_inner()
                .into_iter()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect();
            (flat, w, h)
        }
        Intermediate::Monochrome(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels
                .into_inner()
                .into_iter()
                .flat_map(|v| [v, v, v])
                .collect();
            (flat, w, h)
        }
    };

    Ok(RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 16,
        wb_coeffs,
        xyz_to_cam,
        color_matrices,
        data,
        cpp: 3,
        orientation,
        is_raw: true,
        exif,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputFormat {
    Jpeg,
    Png,
    Tiff,
    Webp,
    Heif,
    Jxl,
    Gif,
    Bmp,
}

fn sniff_format(data: &[u8]) -> Option<InputFormat> {
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

fn decode_image(
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
        orientation: (false, false, false),
        is_raw: false,
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
        orientation: (false, false, false),
        is_raw: false,
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
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
    let lib_heif = LibHeif::new();
    let ctx = HeifContext::read_from_bytes(data)
        .map_err(|e| PipelineError::Decode(format!("heif: {e}")))?;
    let handle = ctx
        .primary_image_handle()
        .map_err(|e| PipelineError::Decode(format!("heif handle: {e}")))?;
    let image = lib_heif
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .map_err(|e| PipelineError::Decode(format!("heif decode: {e}")))?;
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
    Ok(frame_from_rgb8(rgb, width, height, exif))
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

fn shift_cfa(cfa: &str, dx: usize, dy: usize) -> String {
    let b = cfa.as_bytes();
    if b.len() < 4 {
        return cfa.to_string();
    }
    let get = |x: usize, y: usize| -> u8 { b[((y % 2) * 2 + (x % 2)) % 4] };
    String::from_utf8(vec![
        get(dx, dy),
        get(dx + 1, dy),
        get(dx, dy + 1),
        get(dx + 1, dy + 1),
    ])
    .unwrap_or_else(|_| cfa.to_string())
}

#[cfg(test)]
mod sniff_tests {
    use super::*;

    #[test]
    fn sniff_known_magics() {
        let cases: &[(&[u8], InputFormat)] = &[
            (&[0xFF, 0xD8, 0xFF, 0xE0], InputFormat::Jpeg),
            (b"\x89PNG\r\n\x1a\n", InputFormat::Png),
            (b"II*\0", InputFormat::Tiff),
            (b"MM\0*", InputFormat::Tiff),
            (b"RIFF\0\0\0\0WEBP", InputFormat::Webp),
            (b"\0\0\0\x20ftypheic", InputFormat::Heif),
            (b"\0\0\0\x20ftypmif1", InputFormat::Heif),
            (b"\0\0\0\x20ftypavif", InputFormat::Heif),
            (&[0xFF, 0x0A], InputFormat::Jxl),
            (b"\0\0\0\x0CJXL \r\n\x87\n", InputFormat::Jxl),
            (b"GIF87a", InputFormat::Gif),
            (b"GIF89a", InputFormat::Gif),
            (b"BM\0\0", InputFormat::Bmp),
        ];
        for (bytes, expected) in cases {
            if sniff_format(bytes) != Some(*expected) {
                panic!(
                    "sniff failed for {expected:?}: got {:?}",
                    sniff_format(bytes)
                );
            }
        }
    }

    #[test]
    fn sniff_unknown_returns_none() {
        if sniff_format(b"not-an-image").is_some() {
            panic!("unknown bytes should not sniff");
        }
    }
}
