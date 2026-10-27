use super::gain::{GainMap, MapArea, VignetteRadial};
use crate::geom::dng_warp::{DngWarp, WarpPlane};

const MAX_OPCODES: u32 = 1024;
const MAX_BAD_POINTS: u32 = 1 << 20;
const MAX_MAP_POINTS: u64 = 1 << 22;
const MAX_COEFFICIENT: f64 = 1e3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpcodeList {
    One,
    Two,
    Three,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BadPixelList {
    pub points: Vec<[u32; 2]>,
    pub rects: Vec<[u32; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Opcode {
    WarpRectilinear(DngWarp),
    FixVignetteRadial(VignetteRadial),
    FixBadPixelsConstant(u32),
    FixBadPixelsList(BadPixelList),
    GainMap(GainMap),
}

impl Opcode {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::WarpRectilinear(_) => "WarpRectilinear",
            Self::FixVignetteRadial(_) => "FixVignetteRadial",
            Self::FixBadPixelsConstant(_) => "FixBadPixelsConstant",
            Self::FixBadPixelsList(_) => "FixBadPixelsList",
            Self::GainMap(_) => "GainMap",
        }
    }

    fn is_identity(&self) -> bool {
        match self {
            Self::WarpRectilinear(warp) => warp.is_identity(),
            Self::FixVignetteRadial(radial) => radial.is_identity(),
            Self::GainMap(map) => map.is_identity(),
            Self::FixBadPixelsConstant(_) | Self::FixBadPixelsList(_) => false,
        }
    }
}

enum Rejected {
    Unsupported(&'static str),
    Malformed,
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        if len > self.bytes.len() {
            return None;
        }
        let (head, tail) = self.bytes.split_at(len);
        self.bytes = tail;
        Some(head)
    }

    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N).and_then(|b| b.try_into().ok())
    }

    fn u32(&mut self) -> Option<u32> {
        self.array().map(u32::from_be_bytes)
    }

    fn f32(&mut self) -> Option<f32> {
        self.array().map(f32::from_be_bytes)
    }

    fn finite(&mut self) -> Option<f64> {
        self.array()
            .map(f64::from_be_bytes)
            .filter(|v| v.is_finite())
    }

    fn coefficient(&mut self) -> Option<f64> {
        self.finite().filter(|v| v.abs() <= MAX_COEFFICIENT)
    }

    fn many<T: Copy + Default, const N: usize>(
        &mut self,
        read: fn(&mut Self) -> Option<T>,
    ) -> Option<[T; N]> {
        let mut out = [T::default(); N];
        for value in &mut out {
            *value = read(self)?;
        }
        Some(out)
    }

    fn remaining(&self) -> u64 {
        self.bytes.len() as u64
    }
}

pub(super) fn parse(blob: &[u8], list: OpcodeList) -> Vec<Opcode> {
    let mut reader = Reader { bytes: blob };
    let Some(count) = reader.u32() else {
        return Vec::new();
    };
    let mut ops = Vec::new();
    for _ in 0..count.min(MAX_OPCODES) {
        let header = (reader.u32(), reader.u32(), reader.u32(), reader.u32());
        let (Some(id), Some(_version), Some(flags), Some(len)) = header else {
            tracing::warn!(?list, "truncated DNG opcode header");
            break;
        };
        let Some(payload) = reader.take(len as usize) else {
            tracing::warn!(?list, id, len, "truncated DNG opcode payload");
            break;
        };
        match decode(id, payload) {
            Ok(op) if op.is_identity() => {}
            Ok(op) => ops.push(op),
            Err(Rejected::Unsupported(name)) => {
                let optional = flags & 1 != 0;
                tracing::info!(?list, id, name, optional, "skipping unsupported DNG opcode");
            }
            Err(Rejected::Malformed) => tracing::warn!(?list, id, "skipping malformed DNG opcode"),
        }
    }
    ops
}

fn decode(id: u32, payload: &[u8]) -> Result<Opcode, Rejected> {
    let mut reader = Reader { bytes: payload };
    let op = match id {
        1 => warp_rectilinear(&mut reader).map(Opcode::WarpRectilinear),
        3 => vignette_radial(&mut reader).map(Opcode::FixVignetteRadial),
        4 => reader.u32().map(Opcode::FixBadPixelsConstant),
        5 => bad_pixel_list(&mut reader).map(Opcode::FixBadPixelsList),
        9 => gain_map(&mut reader).map(Opcode::GainMap),
        _ => return Err(Rejected::Unsupported(unsupported_name(id))),
    };
    op.ok_or(Rejected::Malformed)
}

fn unsupported_name(id: u32) -> &'static str {
    match id {
        2 => "WarpFisheye",
        6 => "TrimBounds",
        7 => "MapTable",
        8 => "MapPolynomial",
        10 => "DeltaPerRow",
        11 => "DeltaPerColumn",
        12 => "ScalePerRow",
        13 => "ScalePerColumn",
        14 => "WarpRectilinear2",
        _ => "unknown",
    }
}

fn warp_rectilinear(reader: &mut Reader) -> Option<DngWarp> {
    let planes = match reader.u32()? {
        1 => [warp_plane(reader)?; 3],
        3 => [
            warp_plane(reader)?,
            warp_plane(reader)?,
            warp_plane(reader)?,
        ],
        _ => return None,
    };
    let center = reader.many(Reader::coefficient)?.map(|v| v as f32);
    Some(DngWarp { planes, center })
}

fn warp_plane(reader: &mut Reader) -> Option<WarpPlane> {
    let radial = reader.many(Reader::coefficient)?.map(|v| v as f32);
    let tangential = reader.many(Reader::coefficient)?.map(|v| v as f32);
    Some(WarpPlane { radial, tangential })
}

fn vignette_radial(reader: &mut Reader) -> Option<VignetteRadial> {
    let k = reader.many(Reader::coefficient)?;
    let center = reader.many(Reader::coefficient)?;
    Some(VignetteRadial { k, center })
}

fn bad_pixel_list(reader: &mut Reader) -> Option<BadPixelList> {
    let _phase = reader.u32()?;
    let point_count = reader.u32()?;
    let rect_count = reader.u32()?;
    let needed = u64::from(point_count) * 8 + u64::from(rect_count) * 16;
    if point_count > MAX_BAD_POINTS || rect_count > MAX_BAD_POINTS || needed > reader.remaining() {
        return None;
    }
    let points = (0..point_count)
        .map(|_| reader.many(Reader::u32))
        .collect::<Option<Vec<_>>>()?;
    let rects = (0..rect_count)
        .map(|_| reader.many(Reader::u32))
        .collect::<Option<Vec<_>>>()?;
    Some(BadPixelList { points, rects })
}

fn gain_map(reader: &mut Reader) -> Option<GainMap> {
    let [top, left, bottom, right] = reader.many(Reader::u32)?;
    let area = MapArea {
        top,
        left,
        bottom,
        right,
    };
    let [plane, planes] = reader.many(Reader::u32)?;
    let pitch = reader.many(Reader::u32)?;
    let points = reader.many(Reader::u32)?;
    let spacing = reader.many(Reader::finite)?;
    let origin = reader.many(Reader::finite)?;
    let map_planes = reader.u32()?;
    let count = u64::from(points[0]) * u64::from(points[1]) * u64::from(map_planes);
    let valid = area.bottom > area.top
        && area.right > area.left
        && planes > 0
        && pitch.iter().all(|p| *p > 0)
        && count > 0
        && count <= MAX_MAP_POINTS
        && count * 4 <= reader.remaining()
        && points.iter().zip(spacing).all(|(n, s)| *n == 1 || s > 0.0);
    if !valid {
        return None;
    }
    let gains = (0..count)
        .map(|_| {
            reader
                .f32()
                .filter(|g| (0.0..=MAX_COEFFICIENT as f32).contains(g))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(GainMap {
        area,
        plane,
        planes,
        pitch,
        points: points.map(|n| n as usize),
        spacing,
        origin,
        map_planes: map_planes as usize,
        gains,
    })
}

#[cfg(test)]
mod tests;
