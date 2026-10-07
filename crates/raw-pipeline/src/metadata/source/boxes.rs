use super::be_u32;

pub(super) fn boxes(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut pos = 0usize;
    std::iter::from_fn(move || {
        let declared = be_u32(data, pos)?;
        let kind = data.get(pos + 4..pos + 8)?;
        let (header, size) = match declared {
            0 => (8, data.len() - pos),
            1 => (
                16,
                usize::try_from(u64::from_be_bytes(
                    data.get(pos + 8..pos + 16)?.try_into().ok()?,
                ))
                .ok()?,
            ),
            n => (8, n as usize),
        };
        let end = pos.checked_add(size)?;
        let body = data.get(pos + header..end)?;
        pos = end;
        Some((kind, body))
    })
}
