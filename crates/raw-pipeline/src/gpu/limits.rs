use wgpu::Limits;

use crate::{PipelineError, PipelineResult};

pub(crate) fn check_texture(limits: &Limits, label: &str, dims: (u32, u32)) -> PipelineResult<()> {
    let (w, h) = dims;
    let max = limits.max_texture_dimension_2d;
    if w == 0 || h == 0 || w > max || h > max {
        return Err(PipelineError::Unsupported(format!(
            "{label} {w}x{h} outside the gpu texture limit 1..={max}"
        )));
    }
    Ok(())
}

pub(crate) fn check_buffer(limits: &Limits, label: &str, bytes: u64) -> PipelineResult<()> {
    let max = limits.max_buffer_size;
    if bytes > max {
        return Err(PipelineError::Unsupported(format!(
            "{label} needs {bytes} bytes, over the gpu buffer limit {max}"
        )));
    }
    Ok(())
}

#[cfg(feature = "native")]
pub(crate) fn check_storage_buffer(limits: &Limits, label: &str, bytes: u64) -> PipelineResult<()> {
    check_buffer(limits, label, bytes)?;
    let max = limits.max_storage_buffer_binding_size;
    if bytes > max {
        return Err(PipelineError::Unsupported(format!(
            "{label} needs {bytes} bytes, over the gpu storage binding limit {max}"
        )));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn unsupported(result: PipelineResult<()>) -> bool {
    matches!(result, Err(PipelineError::Unsupported(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            max_texture_dimension_2d: 1024,
            max_buffer_size: 1 << 20,
            max_storage_buffer_binding_size: 1 << 16,
            ..Limits::default()
        }
    }

    #[test]
    fn a_texture_fits_up_to_the_limit_on_both_axes() {
        let limits = limits();
        for dims in [(1, 1), (1024, 1), (1, 1024), (1024, 1024)] {
            if let Err(e) = check_texture(&limits, "t", dims) {
                panic!("{dims:?} refused: {e}");
            }
        }
        for dims in [(0, 1), (1, 0), (1025, 1), (1, 1025)] {
            if !unsupported(check_texture(&limits, "t", dims)) {
                panic!("{dims:?} was not refused as unsupported");
            }
        }
    }

    #[test]
    fn a_buffer_fits_up_to_the_buffer_limit() {
        let limits = limits();
        if let Err(e) = check_buffer(&limits, "b", 1 << 20) {
            panic!("a buffer at the limit was refused: {e}");
        }
        if !unsupported(check_buffer(&limits, "b", (1 << 20) + 1)) {
            panic!("a buffer past the limit was not refused");
        }
    }

    #[cfg(feature = "native")]
    #[test]
    fn a_storage_buffer_also_fits_the_binding_limit() {
        let limits = limits();
        if let Err(e) = check_storage_buffer(&limits, "s", 1 << 16) {
            panic!("a storage buffer at the binding limit was refused: {e}");
        }
        if !unsupported(check_storage_buffer(&limits, "s", (1 << 16) + 1)) {
            panic!("a storage buffer past the binding limit was not refused");
        }
    }
}
