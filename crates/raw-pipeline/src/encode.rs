use crate::PipelineError;
use crate::frame::{BitDepth, OutputFormat, PngCompression, TiffCompression};
use libheif_rs::{
    Channel, ColorSpace, CompressionFormat, EncoderQuality, HeifContext, Image as HeifImage,
    LibHeif, RgbChroma,
};

pub struct ImageRgb8<'a> {
    pub rgb: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub struct ImageRgba8<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub fn encode_jpeg_rgb(img: ImageRgb8<'_>, quality: i32) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: img.rgb,
        width: img.width as usize,
        pitch: img.width as usize * 3,
        height: img.height as usize,
        format: turbojpeg::PixelFormat::RGB,
    };
    turbojpeg::compress(image, quality, turbojpeg::Subsamp::Sub2x2)
        .map(|buf| buf.to_vec())
        .map_err(|e| PipelineError::Encode(format!("{e}")))
}

pub fn encode_jpeg_rgba(img: ImageRgba8<'_>, quality: i32) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: img.rgba,
        width: img.width as usize,
        pitch: img.width as usize * 4,
        height: img.height as usize,
        format: turbojpeg::PixelFormat::RGBA,
    };
    turbojpeg::compress(image, quality, turbojpeg::Subsamp::Sub2x2)
        .map(|buf| buf.to_vec())
        .map_err(|e| PipelineError::Encode(format!("{e}")))
}

pub fn encode_png8(
    img: ImageRgb8<'_>,
    compression: PngCompression,
) -> crate::PipelineResult<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, img.width, img.height);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(map_png_compression(compression));
        let mut writer = enc
            .write_header()
            .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
        writer
            .write_image_data(img.rgb)
            .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    }
    Ok(buf)
}

pub fn encode_png16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    compression: PngCompression,
) -> crate::PipelineResult<Vec<u8>> {
    let mut be: Vec<u8> = Vec::with_capacity(rgb16.len() * 2);
    for &v in rgb16 {
        be.extend_from_slice(&v.to_be_bytes());
    }
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, width, height);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Sixteen);
        enc.set_compression(map_png_compression(compression));
        let mut writer = enc
            .write_header()
            .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
        writer
            .write_image_data(&be)
            .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    }
    Ok(buf)
}

fn map_png_compression(c: PngCompression) -> png::Compression {
    match c {
        PngCompression::Fast => png::Compression::Fast,
        PngCompression::Default => png::Compression::Balanced,
        PngCompression::Best => png::Compression::High,
    }
}

pub fn encode_webp_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    lossless: bool,
) -> crate::PipelineResult<Vec<u8>> {
    let encoder = webp::Encoder::from_rgb(img.rgb, img.width, img.height);
    let mem = if lossless {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality as f32)
    };
    Ok(mem.to_vec())
}

fn encode_heif_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    format: CompressionFormat,
) -> crate::PipelineResult<Vec<u8>> {
    let lib_heif = LibHeif::new();
    let mut heif_image = HeifImage::new(img.width, img.height, ColorSpace::Rgb(RgbChroma::Rgb))
        .map_err(|e| PipelineError::Encode(format!("heif image: {e}")))?;
    heif_image
        .create_plane(Channel::Interleaved, img.width, img.height, 8)
        .map_err(|e| PipelineError::Encode(format!("heif plane: {e}")))?;
    {
        let planes = heif_image.planes_mut();
        let plane = planes
            .interleaved
            .ok_or_else(|| PipelineError::Encode("heif: no interleaved plane".into()))?;
        let stride = plane.stride;
        let row_bytes = img.width as usize * 3;
        for y in 0..img.height as usize {
            let dst_off = y * stride;
            let src_off = y * row_bytes;
            plane.data[dst_off..dst_off + row_bytes]
                .copy_from_slice(&img.rgb[src_off..src_off + row_bytes]);
        }
    }
    let mut encoder = lib_heif
        .encoder_for_format(format)
        .map_err(|e| PipelineError::Encode(format!("heif encoder: {e}")))?;
    encoder
        .set_quality(EncoderQuality::Lossy(quality))
        .map_err(|e| PipelineError::Encode(format!("heif quality: {e}")))?;
    let mut ctx =
        HeifContext::new().map_err(|e| PipelineError::Encode(format!("heif ctx: {e}")))?;
    ctx.encode_image(&heif_image, &mut encoder, None)
        .map_err(|e| PipelineError::Encode(format!("heif encode: {e}")))?;
    ctx.write_to_bytes()
        .map_err(|e| PipelineError::Encode(format!("heif write: {e}")))
}

pub fn encode_avif_rgb(img: ImageRgb8<'_>, quality: u8) -> crate::PipelineResult<Vec<u8>> {
    encode_heif_rgb(img, quality, CompressionFormat::Av1)
}

pub fn encode_heic_rgb(img: ImageRgb8<'_>, quality: u8) -> crate::PipelineResult<Vec<u8>> {
    encode_heif_rgb(img, quality, CompressionFormat::Hevc)
}

pub fn encode_tiff8(
    img: ImageRgb8<'_>,
    compression: TiffCompression,
) -> crate::PipelineResult<Vec<u8>> {
    use tiff::encoder::colortype;
    let mut buf: Vec<u8> = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut enc = build_tiff_encoder(cursor, compression)?;
        enc.write_image::<colortype::RGB8>(img.width, img.height, img.rgb)
            .map_err(|e| PipelineError::Encode(format!("tiff: {e}")))?;
    }
    Ok(buf)
}

pub fn encode_tiff16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    compression: TiffCompression,
) -> crate::PipelineResult<Vec<u8>> {
    use tiff::encoder::colortype;
    let mut buf: Vec<u8> = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut enc = build_tiff_encoder(cursor, compression)?;
        enc.write_image::<colortype::RGB16>(width, height, rgb16)
            .map_err(|e| PipelineError::Encode(format!("tiff: {e}")))?;
    }
    Ok(buf)
}

fn build_tiff_encoder<W: std::io::Write + std::io::Seek>(
    writer: W,
    compression: TiffCompression,
) -> crate::PipelineResult<tiff::encoder::TiffEncoder<W>> {
    use tiff::encoder::{Compression, TiffEncoder};
    let enc = TiffEncoder::new(writer).map_err(|e| PipelineError::Encode(format!("tiff: {e}")))?;
    let c = match compression {
        TiffCompression::None => Compression::Uncompressed,
        TiffCompression::Lzw => Compression::Lzw,
        TiffCompression::Deflate => Compression::Deflate(tiff::encoder::DeflateLevel::default()),
    };
    Ok(enc.with_compression(c))
}

pub fn encode_jxl8(img: ImageRgb8<'_>) -> crate::PipelineResult<Vec<u8>> {
    let mut encoder = jpegxl_rs::encoder_builder()
        .build()
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    let result: jpegxl_rs::encode::EncoderResult<u8> = encoder
        .encode::<u8, u8>(img.rgb, img.width, img.height)
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    Ok(result.data)
}

pub fn encode_jxl16(rgb16: &[u16], width: u32, height: u32) -> crate::PipelineResult<Vec<u8>> {
    let mut encoder = jpegxl_rs::encoder_builder()
        .build()
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    let result: jpegxl_rs::encode::EncoderResult<u16> = encoder
        .encode::<u16, u16>(rgb16, width, height)
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    Ok(result.data)
}

pub fn encode_from_rgb8(
    rgb: &[u8],
    width: u32,
    height: u32,
    format: &OutputFormat,
) -> crate::PipelineResult<Vec<u8>> {
    let img = ImageRgb8 { rgb, width, height };
    match *format {
        OutputFormat::Jpeg { quality } => encode_jpeg_rgb(img, quality as i32),
        OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression,
        } => encode_png8(img, compression),
        OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_png16(&rgb16, width, height, compression)
        }
        OutputFormat::Webp { quality, lossless } => encode_webp_rgb(img, quality, lossless),
        OutputFormat::Avif { quality } => encode_avif_rgb(img, quality),
        OutputFormat::Heic { quality } => encode_heic_rgb(img, quality),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Eight,
            compression,
        } => encode_tiff8(img, compression),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_tiff16(&rgb16, width, height, compression)
        }
        OutputFormat::Jxl {
            bit_depth: BitDepth::Eight,
        } => encode_jxl8(img),
        OutputFormat::Jxl {
            bit_depth: BitDepth::Sixteen,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_jxl16(&rgb16, width, height)
        }
    }
}

pub fn encode_from_rgba8(
    rgba: &[u8],
    width: u32,
    height: u32,
    format: &OutputFormat,
) -> crate::PipelineResult<Vec<u8>> {
    if let OutputFormat::Jpeg { quality } = *format {
        return encode_jpeg_rgba(
            ImageRgba8 {
                rgba,
                width,
                height,
            },
            quality as i32,
        );
    }
    let mut rgb: Vec<u8> = Vec::with_capacity((width as usize) * (height as usize) * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    encode_from_rgb8(&rgb, width, height, format)
}

pub fn encode_from_rgb16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    format: &OutputFormat,
) -> crate::PipelineResult<Vec<u8>> {
    match *format {
        OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => encode_png16(rgb16, width, height, compression),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => encode_tiff16(rgb16, width, height, compression),
        OutputFormat::Jxl {
            bit_depth: BitDepth::Sixteen,
        } => encode_jxl16(rgb16, width, height),
        _ => {
            let rgb8: Vec<u8> = rgb16.iter().map(|&v| (v >> 8) as u8).collect();
            encode_from_rgb8(&rgb8, width, height, format)
        }
    }
}
