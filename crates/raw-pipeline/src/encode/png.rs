use std::borrow::Cow;
use std::io::Write;

use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::{OutputColorSpace, PngCompression};
use crate::metadata::exif::ExifBlock;

const IDAT_BYTES: usize = 256 * 1024;

struct Header<'a> {
    width: u32,
    height: u32,
    depth: png::BitDepth,
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&'a ExifBlock>,
}

pub fn encode_png8(
    img: ImageRgb8<'_>,
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let header = Header {
        width: img.width,
        height: img.height,
        depth: png::BitDepth::Eight,
        compression,
        cs,
        exif,
    };
    write(header, img.rgb.len(), |out| out.write_all(img.rgb))
}

pub fn encode_png16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let header = Header {
        width,
        height,
        depth: png::BitDepth::Sixteen,
        compression,
        cs,
        exif,
    };
    let samples = width as usize * 3;
    write(header, size_of_val(rgb16), |out| {
        let mut row = vec![0u8; samples * 2];
        for line in rgb16.chunks_exact(samples) {
            for (dst, v) in row.as_chunks_mut::<2>().0.iter_mut().zip(line) {
                dst.copy_from_slice(&v.to_be_bytes());
            }
            out.write_all(&row)?;
        }
        Ok(())
    })
}

fn write(
    header: Header<'_>,
    pixel_bytes: usize,
    pixels: impl FnOnce(&mut dyn Write) -> std::io::Result<()>,
) -> crate::PipelineResult<Vec<u8>> {
    let failed = |e: &dyn std::fmt::Display| PipelineError::Encode(format!("png: {e}"));
    let mut info = png::Info::with_size(header.width, header.height);
    info.color_type = png::ColorType::Rgb;
    info.bit_depth = header.depth;
    info.exif_metadata = header.exif.map(|e| Cow::Borrowed(e.tiff.as_slice()));
    if header.cs != OutputColorSpace::SRgb {
        info.icc_profile = Some(Cow::Borrowed(header.cs.icc_profile()));
    }
    let expands = header.compression == PngCompression::Fast;
    let mut buf = Vec::with_capacity(super::output_capacity(pixel_bytes, expands));
    let mut enc = png::Encoder::with_info(&mut buf, info).map_err(|e| failed(&e))?;
    enc.set_compression(map_png_compression(header.compression));
    if header.cs == OutputColorSpace::SRgb {
        enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    }
    let mut writer = enc.write_header().map_err(|e| failed(&e))?;
    let mut stream = writer
        .stream_writer_with_size(IDAT_BYTES)
        .map_err(|e| failed(&e))?;
    pixels(&mut stream).map_err(|e| failed(&e))?;
    stream.finish().map_err(|e| failed(&e))?;
    writer.finish().map_err(|e| failed(&e))?;
    Ok(buf)
}

fn map_png_compression(c: PngCompression) -> png::Compression {
    match c {
        PngCompression::Fast => png::Compression::Fast,
        PngCompression::Default => png::Compression::Balanced,
        PngCompression::Best => png::Compression::High,
    }
}
