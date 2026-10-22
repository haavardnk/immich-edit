use std::io::Read;

use super::DenoisePlane;
use crate::source::wire::{from_planes, to_planes, wire_pixels};
use crate::{PipelineError, PipelineResult};

const MAGIC: &[u8; 4] = b"IEDN";
const VERSION: u32 = 1;
const HEADER_BYTES: usize = 24;
const ZSTD_LEVEL: i32 = 3;

pub fn encode(plane: &DenoisePlane) -> PipelineResult<Vec<u8>> {
    let (width, height) = (plane.width, plane.height);
    let pixels = wire_pixels(width as u64, height as u64);
    if pixels.is_none_or(|pixels| plane.rgb_f16.len() != pixels * 3) {
        return Err(PipelineError::Encode(format!(
            "denoise plane holds {} samples at {width}x{height}",
            plane.rgb_f16.len()
        )));
    }
    let planes = to_planes(&plane.rgb_f16, width, height);
    let payload = zstd::bulk::compress(&planes, ZSTD_LEVEL)
        .map_err(|e| PipelineError::Encode(format!("denoise plane: {e}")))?;
    let mut out = Vec::with_capacity(HEADER_BYTES + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(plane.width as u32).to_le_bytes());
    out.extend_from_slice(&(plane.height as u32).to_le_bytes());
    out.extend_from_slice(&plane.key.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

pub fn decode(bytes: &[u8]) -> PipelineResult<DenoisePlane> {
    let invalid = |why: &str| PipelineError::Decode(format!("denoise plane: {why}"));
    if bytes.len() < HEADER_BYTES || &bytes[..4] != MAGIC {
        return Err(invalid("not a denoise plane"));
    }
    let word =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    if word(4) != VERSION {
        return Err(invalid(&format!("unsupported version {}", word(4))));
    }
    let width = word(8) as usize;
    let height = word(12) as usize;
    let Some(pixels) = wire_pixels(word(8).into(), word(12).into()) else {
        return Err(invalid(&format!("dims {width}x{height} out of range")));
    };
    let key = u64::from(word(16)) | (u64::from(word(20)) << 32);
    let expected = pixels * 6;
    let mut planes = Vec::with_capacity(expected);
    zstd::stream::read::Decoder::new(&bytes[HEADER_BYTES..])
        .map_err(|e| invalid(&e.to_string()))?
        .take(expected as u64 + 1)
        .read_to_end(&mut planes)
        .map_err(|e| invalid(&e.to_string()))?;
    if planes.len() != expected {
        return Err(invalid("payload size does not match dims"));
    }
    DenoisePlane::new(key, width, height, from_planes(&planes, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane() -> DenoisePlane {
        let rgb: Vec<f32> = (0..7 * 5 * 3).map(|i| i as f32 * 0.01 - 0.2).collect();
        DenoisePlane::from_rgb(0x0123_4567_dead_beef, 7, 5, &rgb).unwrap()
    }

    #[test]
    fn a_plane_survives_the_round_trip_bit_for_bit() {
        let original = plane();
        assert_eq!(decode(&encode(&original).unwrap()).unwrap(), original);
    }

    #[test]
    fn damaged_streams_are_rejected() {
        let bytes = encode(&plane()).unwrap();
        assert!(decode(&bytes[..bytes.len() - 3]).is_err());
        assert!(decode(b"nope").is_err());
        let mut huge = bytes.clone();
        huge[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&huge).is_err());
        let mut wrong = bytes;
        wrong[4] = 9;
        assert!(decode(&wrong).is_err());
    }

    fn restamped(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = encode(&plane()).unwrap();
        bytes[8..12].copy_from_slice(&width.to_le_bytes());
        bytes[12..16].copy_from_slice(&height.to_le_bytes());
        bytes
    }

    #[test]
    fn full_sensor_planes_pass_the_header_check_and_oversized_ones_do_not() {
        let at_target = format!("{:?}", decode(&restamped(12000, 9000)).err());
        assert!(at_target.contains("payload size"), "{at_target}");
        let gfx = format!("{:?}", decode(&restamped(11648, 8736)).err());
        assert!(gfx.contains("payload size"), "{gfx}");
        let over = format!("{:?}", decode(&restamped(16384, 16384)).err());
        assert!(over.contains("out of range"), "{over}");
    }

    #[test]
    fn encoding_refuses_oversized_or_inconsistent_planes_before_allocating() {
        let huge = DenoisePlane {
            key: 1,
            width: 16384,
            height: 16384,
            rgb_f16: Vec::new(),
        };
        assert!(encode(&huge).is_err());
        let short = DenoisePlane {
            rgb_f16: vec![0; 5],
            ..plane()
        };
        assert!(encode(&short).is_err());
    }
}
