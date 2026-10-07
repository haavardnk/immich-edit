use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::OutputColorSpace;
use crate::metadata::exif::ExifBlock;

const ICC_FLAG: u8 = 0x20;
const EXIF_FLAG: u8 = 0x08;

pub fn encode_webp_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    lossless: bool,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let encoder = webp::Encoder::from_rgb(img.rgb, img.width, img.height);
    let mem = if lossless {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality as f32)
    };
    extended(&mem, img.width, img.height, cs.icc_profile(), exif)
}

fn extended(
    webp: &[u8],
    width: u32,
    height: u32,
    icc: &[u8],
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    if webp.get(..4) != Some(b"RIFF") || webp.get(8..12) != Some(b"WEBP") {
        return Err(PipelineError::Encode(
            "webp: encoder output is not RIFF/WEBP".into(),
        ));
    }
    let mut flags = ICC_FLAG;
    let mut image = Vec::new();
    for (kind, body) in chunks(webp) {
        match kind {
            b"VP8X" => flags |= body.first().copied().unwrap_or(0),
            b"ICCP" | b"EXIF" => {}
            _ => chunk(&mut image, kind, body),
        }
    }
    if image.is_empty() {
        return Err(PipelineError::Encode(
            "webp: encoder output has no image chunk".into(),
        ));
    }
    let exif = exif.map(|e| e.tiff.as_slice());
    if exif.is_some() {
        flags |= EXIF_FLAG;
    }
    let mut vp8x = vec![flags, 0, 0, 0];
    vp8x.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
    vp8x.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
    let mut body = b"WEBP".to_vec();
    chunk(&mut body, b"VP8X", &vp8x);
    chunk(&mut body, b"ICCP", icc);
    body.extend_from_slice(&image);
    if let Some(exif) = exif {
        chunk(&mut body, b"EXIF", exif);
    }
    let size = u32::try_from(body.len())
        .map_err(|_| PipelineError::Encode("webp: output exceeds 4 GB".into()))?;
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

fn chunks(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut pos = 12usize;
    std::iter::from_fn(move || {
        let kind = data.get(pos..pos + 4)?;
        let len = u32::from_le_bytes(data.get(pos + 4..pos + 8)?.try_into().ok()?) as usize;
        let body = data.get(pos + 8..(pos + 8).checked_add(len)?)?;
        pos += 8 + len + (len & 1);
        Some((kind, body))
    })
}

fn chunk(out: &mut Vec<u8>, kind: &[u8], body: &[u8]) {
    out.extend_from_slice(kind);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body);
    if body.len() % 2 == 1 {
        out.push(0);
    }
}
