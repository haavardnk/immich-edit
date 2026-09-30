use std::borrow::Cow;
use std::io::Read;
#[cfg(feature = "native")]
use std::io::Write;

#[cfg(feature = "native")]
use rayon::prelude::*;

#[cfg(feature = "native")]
use super::SourceCoding;
use super::{SourceImage, header};
use crate::{PipelineError, PipelineResult};

const MAGIC: &[u8; 4] = b"IESR";
const PLAIN_MAGIC: &[u8; 4] = b"IESP";
const VERSION: u32 = 2;
const PREFIX_BYTES: usize = 12;
const MAX_HEADER_BYTES: usize = 4096;
const MAX_PIXELS: u64 = 64 * 1024 * 1024;
#[cfg(feature = "native")]
const ZSTD_LEVEL: i32 = 1;

#[cfg(feature = "native")]
pub fn encode(image: &SourceImage, coding: SourceCoding) -> PipelineResult<Vec<u8>> {
    let (w, h) = image.header.dims;
    if image.rgb_f16.len() as u64 != w as u64 * h as u64 * 3 {
        return Err(PipelineError::Encode(format!(
            "source holds {} samples, expected {w}x{h}x3",
            image.rgb_f16.len()
        )));
    }
    let mut head = Vec::new();
    header::write(&image.header, &mut head)?;
    let planes = to_planes(&image.rgb_f16, w as usize, h as usize);
    match coding {
        SourceCoding::Framed => Ok(assemble(MAGIC, &head, &compress(&[&planes])?)),
        SourceCoding::ZstdContent => compress(&[&prefix(PLAIN_MAGIC, head.len()), &head, &planes]),
    }
}

#[cfg(feature = "native")]
fn compress(parts: &[&[u8]]) -> PipelineResult<Vec<u8>> {
    let encode_err = |e: std::io::Error| PipelineError::Encode(format!("source payload: {e}"));
    let mut encoder = zstd::stream::Encoder::new(Vec::new(), ZSTD_LEVEL).map_err(encode_err)?;
    encoder
        .multithread(rayon::current_num_threads() as u32)
        .map_err(encode_err)?;
    for part in parts {
        encoder.write_all(part).map_err(encode_err)?;
    }
    encoder.finish().map_err(encode_err)
}

#[cfg(feature = "native")]
fn prefix(magic: &[u8; 4], head_len: usize) -> [u8; PREFIX_BYTES] {
    let mut out = [0u8; PREFIX_BYTES];
    out[..4].copy_from_slice(magic);
    out[4..8].copy_from_slice(&VERSION.to_le_bytes());
    out[8..].copy_from_slice(&(head_len as u32).to_le_bytes());
    out
}

#[cfg(feature = "native")]
fn assemble(magic: &[u8; 4], head: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PREFIX_BYTES + head.len() + payload.len());
    out.extend_from_slice(&prefix(magic, head.len()));
    out.extend_from_slice(head);
    out.extend_from_slice(payload);
    out
}

pub fn decode(bytes: &[u8]) -> PipelineResult<SourceImage> {
    let invalid = |why: &str| PipelineError::Decode(format!("source: {why}"));
    if bytes.len() < PREFIX_BYTES {
        return Err(invalid("not a source stream"));
    }
    let compressed = match &bytes[..4] {
        magic if magic == MAGIC => true,
        magic if magic == PLAIN_MAGIC => false,
        _ => return Err(invalid("not a source stream")),
    };
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != VERSION {
        return Err(invalid(&format!("unsupported version {version}")));
    }
    let head_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if head_len > MAX_HEADER_BYTES || head_len > bytes.len() - PREFIX_BYTES {
        return Err(invalid("header length out of range"));
    }
    let (head, payload) = bytes[PREFIX_BYTES..].split_at(head_len);
    let header = header::read(head)?;
    let (w, h) = header.dims;
    let pixels = w as u64 * h as u64;
    if pixels == 0 || pixels > MAX_PIXELS {
        return Err(invalid(&format!("dims {w}x{h} out of range")));
    }
    let expected = pixels as usize * 6;
    let planes = if compressed {
        let mut planes = Vec::with_capacity(expected);
        ruzstd::decoding::StreamingDecoder::new(payload)
            .map_err(|e| invalid(&format!("payload: {e}")))?
            .take(expected as u64 + 1)
            .read_to_end(&mut planes)
            .map_err(|e| invalid(&format!("payload: {e}")))?;
        Cow::Owned(planes)
    } else {
        Cow::Borrowed(payload)
    };
    if planes.len() != expected {
        return Err(invalid("payload size does not match dims"));
    }
    Ok(SourceImage {
        header,
        rgb_f16: from_planes(&planes, w as usize, h as usize),
    })
}

#[cfg(feature = "native")]
fn to_planes(rgb: &[u16], w: usize, h: usize) -> Vec<u8> {
    let samples = w * h * 3;
    let mut planes = vec![0u8; samples * 2];
    let (hi, lo) = planes.split_at_mut(samples);
    hi.par_chunks_mut(w.max(1))
        .zip(lo.par_chunks_mut(w.max(1)))
        .enumerate()
        .for_each(|(i, (hi_row, lo_row))| {
            let c = i / h;
            let y = i % h;
            let row = &rgb[y * w * 3..(y + 1) * w * 3];
            let mut prev = 0u16;
            for ((px, high), low) in row.chunks_exact(3).zip(hi_row).zip(lo_row) {
                [*high, *low] = px[c].wrapping_sub(prev).to_be_bytes();
                prev = px[c];
            }
        });
    planes
}

fn from_planes(planes: &[u8], w: usize, h: usize) -> Vec<u16> {
    let samples = w * h * 3;
    let (hi, lo) = planes.split_at(samples);
    let mut rgb = vec![0u16; samples];
    for (y, row) in rgb.chunks_exact_mut(w * 3).enumerate() {
        let rows: [(&[u8], &[u8]); 3] = std::array::from_fn(|c| {
            let start = (c * h + y) * w;
            (&hi[start..start + w], &lo[start..start + w])
        });
        let mut prev = [0u16; 3];
        for (x, px) in row.chunks_exact_mut(3).enumerate() {
            prev = std::array::from_fn(|c| {
                prev[c].wrapping_add(u16::from_be_bytes([rows[c].0[x], rows[c].1[x]]))
            });
            px.copy_from_slice(&prev);
        }
    }
    rgb
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;
    use crate::frame::FrameMeta;
    use crate::source::{LinearKind, SourceHeader, SourceWindow};

    fn image(w: u32, h: u32) -> SourceImage {
        SourceImage {
            header: SourceHeader {
                meta: FrameMeta {
                    width: 6000,
                    height: 4000,
                    wb_coeffs: [2.0, 1.0, 1.5, f32::NAN],
                    xyz_to_cam: [[0.5; 3], [0.25; 3], [-0.125; 3], [f32::NAN; 3]],
                    color_matrices: vec![(2856.0, [[0.75; 3]; 4]), (6504.0, [[0.25; 3]; 4])],
                    orientation: (true, false, true),
                    is_raw: true,
                    capture_sigma: Some(0.7),
                    model: "Camera Ø".into(),
                },
                kind: LinearKind::PostWb,
                dims: (w, h),
                atmosphere: Some([0.9, 0.8, 0.7]),
                window: Some(SourceWindow {
                    origin: (128, 256),
                    full: (6000, 4000),
                }),
            },
            rgb_f16: (0..w * h * 3)
                .map(|i| half::f16::from_f32((i as f32 * 0.37).sin() * 4.0).to_bits())
                .collect(),
        }
    }

    #[test]
    fn a_source_round_trips_bit_for_bit() {
        let original = image(37, 11);
        let bytes = encode(&original, SourceCoding::Framed).unwrap();
        let decoded = decode(&bytes).unwrap();
        if decoded.rgb_f16 != original.rgb_f16
            || encode(&decoded, SourceCoding::Framed).unwrap() != bytes
        {
            panic!("source changed across the wire");
        }
        if !decoded.header.meta.xyz_to_cam[3][0].is_nan() {
            panic!("a NaN matrix entry did not survive the wire");
        }
    }

    #[test]
    fn zstd_content_decodes_to_the_same_source_within_the_http_window() {
        let original = image(256, 128);
        let body = encode(&original, SourceCoding::ZstdContent).unwrap();
        let descriptor = body[4];
        if descriptor & 0x20 != 0 {
            panic!("single-segment frame: the window equals the content size");
        }
        let exponent = u32::from(body[5] >> 3);
        let base = 1u64 << (10 + exponent);
        let window = base + base / 8 * u64::from(body[5] & 7);
        if window > 8 << 20 {
            panic!("{window} byte window exceeds the 8 MiB HTTP zstd limit (RFC 9659)");
        }
        let mut plain = Vec::new();
        ruzstd::decoding::StreamingDecoder::new(body.as_slice())
            .unwrap()
            .read_to_end(&mut plain)
            .unwrap();
        let decoded = decode(&plain).unwrap();
        if decoded.rgb_f16 != original.rgb_f16 || decoded.header.dims != original.header.dims {
            panic!("zstd content changed the source");
        }
        if decode(&plain[..plain.len() - 1]).is_ok() {
            panic!("a truncated plain payload decoded");
        }
    }

    #[test]
    fn malformed_streams_are_rejected() {
        let source = image(8, 4);
        let good = encode(&source, SourceCoding::Framed).unwrap();
        let head_len = u32::from_le_bytes([good[8], good[9], good[10], good[11]]) as usize;
        let payload = &good[PREFIX_BYTES + head_len..];
        let restamp = |dims: (u32, u32)| {
            let mut header = source.header.clone();
            header.dims = dims;
            let mut head = Vec::new();
            header::write(&header, &mut head).unwrap();
            assemble(MAGIC, &head, payload)
        };
        let outside = {
            let mut header = source.header.clone();
            header.window = Some(SourceWindow {
                origin: (5995, 0),
                full: (6000, 4000),
            });
            let mut head = Vec::new();
            header::write(&header, &mut head).unwrap();
            assemble(MAGIC, &head, payload)
        };
        let mut bad_magic = good.clone();
        bad_magic[0] = b'X';
        let mut bad_version = good.clone();
        bad_version[4] = 9;
        let mut huge_header = good.clone();
        huge_header[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        let mut short_header = good.clone();
        short_header[8..12].copy_from_slice(&((head_len - 1) as u32).to_le_bytes());
        for (label, bytes) in [
            ("empty", Vec::new()),
            ("magic", bad_magic),
            ("version", bad_version),
            ("header length", huge_header),
            ("short header", short_header),
            ("truncated payload", good[..good.len() - 4].to_vec()),
            ("taller than the payload", restamp((8, 5))),
            ("zero dims", restamp((0, 4))),
            ("window outside the frame", outside),
        ] {
            if decode(&bytes).is_ok() {
                panic!("{label}: a malformed stream decoded");
            }
        }
    }
}
