use std::io::Read;

use super::{SourceImage, header};
use crate::{PipelineError, PipelineResult};

const MAGIC: &[u8; 4] = b"IESR";
const VERSION: u32 = 2;
const PREFIX_BYTES: usize = 12;
const MAX_HEADER_BYTES: usize = 4096;
const MAX_PIXELS: u64 = 64 * 1024 * 1024;
#[cfg(feature = "native")]
const ZSTD_LEVEL: i32 = 1;

#[cfg(feature = "native")]
pub fn encode(image: &SourceImage) -> PipelineResult<Vec<u8>> {
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
    let payload = zstd::encode_all(planes.as_slice(), ZSTD_LEVEL)
        .map_err(|e| PipelineError::Encode(format!("source payload: {e}")))?;
    Ok(assemble(&head, &payload))
}

#[cfg(feature = "native")]
fn assemble(head: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PREFIX_BYTES + head.len() + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(head.len() as u32).to_le_bytes());
    out.extend_from_slice(head);
    out.extend_from_slice(payload);
    out
}

pub fn decode(bytes: &[u8]) -> PipelineResult<SourceImage> {
    let invalid = |why: &str| PipelineError::Decode(format!("source: {why}"));
    if bytes.len() < PREFIX_BYTES || &bytes[..4] != MAGIC {
        return Err(invalid("not a source stream"));
    }
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
    let mut planes = Vec::with_capacity(expected);
    ruzstd::decoding::StreamingDecoder::new(payload)
        .map_err(|e| invalid(&format!("payload: {e}")))?
        .take(expected as u64 + 1)
        .read_to_end(&mut planes)
        .map_err(|e| invalid(&format!("payload: {e}")))?;
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
    for (i, (c, y)) in (0..3).flat_map(|c| (0..h).map(move |y| (c, y))).enumerate() {
        let row = &rgb[y * w * 3..(y + 1) * w * 3];
        let mut prev = 0u16;
        for (x, px) in row.chunks_exact(3).enumerate() {
            let [high, low] = px[c].wrapping_sub(prev).to_be_bytes();
            prev = px[c];
            hi[i * w + x] = high;
            lo[i * w + x] = low;
        }
    }
    planes
}

fn from_planes(planes: &[u8], w: usize, h: usize) -> Vec<u16> {
    let samples = w * h * 3;
    let (hi, lo) = planes.split_at(samples);
    let mut rgb = vec![0u16; samples];
    for (i, (c, y)) in (0..3).flat_map(|c| (0..h).map(move |y| (c, y))).enumerate() {
        let row = &mut rgb[y * w * 3..(y + 1) * w * 3];
        let mut prev = 0u16;
        for (x, px) in row.chunks_exact_mut(3).enumerate() {
            prev = prev.wrapping_add(u16::from_be_bytes([hi[i * w + x], lo[i * w + x]]));
            px[c] = prev;
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
        let bytes = encode(&original).unwrap();
        let decoded = decode(&bytes).unwrap();
        if decoded.rgb_f16 != original.rgb_f16 || encode(&decoded).unwrap() != bytes {
            panic!("source changed across the wire");
        }
        if !decoded.header.meta.xyz_to_cam[3][0].is_nan() {
            panic!("a NaN matrix entry did not survive the wire");
        }
    }

    #[test]
    fn malformed_streams_are_rejected() {
        let source = image(8, 4);
        let good = encode(&source).unwrap();
        let head_len = u32::from_le_bytes([good[8], good[9], good[10], good[11]]) as usize;
        let payload = &good[PREFIX_BYTES + head_len..];
        let restamp = |dims: (u32, u32)| {
            let mut header = source.header.clone();
            header.dims = dims;
            let mut head = Vec::new();
            header::write(&header, &mut head).unwrap();
            assemble(&head, payload)
        };
        let outside = {
            let mut header = source.header.clone();
            header.window = Some(SourceWindow {
                origin: (5995, 0),
                full: (6000, 4000),
            });
            let mut head = Vec::new();
            header::write(&header, &mut head).unwrap();
            assemble(&head, payload)
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
