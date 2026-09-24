use super::{LinearKind, SourceHeader};
use crate::frame::FrameMeta;
use crate::{PipelineError, PipelineResult};

const MAX_COLOR_MATRICES: usize = 8;

#[cfg(feature = "native")]
pub(super) fn write(header: &SourceHeader, out: &mut Vec<u8>) -> PipelineResult<()> {
    let meta = &header.meta;
    let too_long = |what: &str| PipelineError::Encode(format!("source header: {what} too long"));
    let matrices = u8::try_from(meta.color_matrices.len())
        .ok()
        .filter(|n| *n as usize <= MAX_COLOR_MATRICES)
        .ok_or_else(|| too_long("colour matrix list"))?;
    let model_len = u8::try_from(meta.model.len()).map_err(|_| too_long("model"))?;
    put_u32(out, meta.width as u32);
    put_u32(out, meta.height as u32);
    put_f32s(out, &meta.wb_coeffs);
    put_f32s(out, meta.xyz_to_cam.as_flattened());
    out.push(matrices);
    for (illuminant, matrix) in &meta.color_matrices {
        put_f32s(out, &[*illuminant]);
        put_f32s(out, matrix.as_flattened());
    }
    let (transpose, flip_h, flip_v) = meta.orientation;
    out.extend([transpose, flip_h, flip_v, meta.is_raw].map(u8::from));
    put_optional(out, meta.capture_sigma.map(|s| [s]).as_ref());
    out.push(model_len);
    out.extend_from_slice(meta.model.as_bytes());
    out.push(match header.kind {
        LinearKind::PreWb => 0,
        LinearKind::PostWb => 1,
    });
    put_u32(out, header.dims.0);
    put_u32(out, header.dims.1);
    put_optional(out, header.atmosphere.as_ref());
    Ok(())
}

pub(super) fn read(bytes: &[u8]) -> PipelineResult<SourceHeader> {
    let mut r = Reader { bytes, pos: 0 };
    let width = r.u32()? as usize;
    let height = r.u32()? as usize;
    let wb_coeffs = r.f32s::<4>()?;
    let xyz_to_cam = rows(r.f32s::<12>()?);
    let count = r.u8()? as usize;
    if count > MAX_COLOR_MATRICES {
        return Err(invalid("too many colour matrices"));
    }
    let color_matrices = (0..count)
        .map(|_| Ok((r.f32s::<1>()?[0], rows(r.f32s::<12>()?))))
        .collect::<PipelineResult<Vec<_>>>()?;
    let orientation = (r.flag()?, r.flag()?, r.flag()?);
    let is_raw = r.flag()?;
    let capture_sigma = r.optional::<1>()?.map(|[s]| s);
    let model_len = r.u8()? as usize;
    let model = String::from_utf8(r.take(model_len)?.to_vec())
        .map_err(|_| invalid("model is not utf-8"))?;
    let kind = match r.u8()? {
        0 => LinearKind::PreWb,
        1 => LinearKind::PostWb,
        other => return Err(invalid(&format!("unknown source kind {other}"))),
    };
    let dims = (r.u32()?, r.u32()?);
    let atmosphere = r.optional::<3>()?;
    if r.pos != bytes.len() {
        return Err(invalid("trailing header bytes"));
    }
    Ok(SourceHeader {
        meta: FrameMeta {
            width,
            height,
            wb_coeffs,
            xyz_to_cam,
            color_matrices,
            orientation,
            is_raw,
            capture_sigma,
            model,
        },
        kind,
        dims,
        atmosphere,
    })
}

fn invalid(why: &str) -> PipelineError {
    PipelineError::Decode(format!("source header: {why}"))
}

fn rows(flat: [f32; 12]) -> [[f32; 3]; 4] {
    [0, 1, 2, 3].map(|r| [flat[r * 3], flat[r * 3 + 1], flat[r * 3 + 2]])
}

#[cfg(feature = "native")]
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

#[cfg(feature = "native")]
fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    out.extend(values.iter().flat_map(|v| v.to_bits().to_le_bytes()));
}

#[cfg(feature = "native")]
fn put_optional<const N: usize>(out: &mut Vec<u8>, values: Option<&[f32; N]>) {
    out.push(u8::from(values.is_some()));
    if let Some(values) = values {
        put_f32s(out, values);
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> PipelineResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid("truncated"))?;
        let out = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn u8(&mut self) -> PipelineResult<u8> {
        Ok(self.take(1)?[0])
    }

    fn flag(&mut self) -> PipelineResult<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("flag is not 0 or 1")),
        }
    }

    fn u32(&mut self) -> PipelineResult<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn f32s<const N: usize>(&mut self) -> PipelineResult<[f32; N]> {
        let mut out = [0.0; N];
        for v in &mut out {
            *v = f32::from_bits(self.u32()?);
        }
        Ok(out)
    }

    fn optional<const N: usize>(&mut self) -> PipelineResult<Option<[f32; N]>> {
        match self.flag()? {
            true => self.f32s::<N>().map(Some),
            false => Ok(None),
        }
    }
}
