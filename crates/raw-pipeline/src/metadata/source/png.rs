use little_exif::metadata::Metadata;

use super::{exif_block, read_capped};

const EXIF_PROFILES: [&[u8]; 2] = [b"Raw profile type exif", b"Raw profile type APP1"];

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    let mut exif: Option<Metadata> = None;
    let mut exif_profile: Option<Metadata> = None;
    for (kind, body) in chunks(data) {
        if kind == b"eXIf" {
            exif = exif.or_else(|| exif_block(body));
            continue;
        }
        let Some((keyword, text)) = text_chunk(kind, body) else {
            continue;
        };
        if EXIF_PROFILES.contains(&keyword) {
            exif_profile = exif_profile.or_else(|| raw_profile(&text).and_then(|b| exif_block(&b)));
        }
    }
    exif.or(exif_profile)
}

fn chunks(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut pos = 8usize;
    std::iter::from_fn(move || {
        let len = super::be_u32(data, pos)? as usize;
        let kind = data.get(pos + 4..pos + 8)?;
        let body = data.get(pos + 8..(pos + 8).checked_add(len)?)?;
        pos += 12 + len;
        Some((kind, body))
    })
}

fn text_chunk<'a>(kind: &[u8], body: &'a [u8]) -> Option<(&'a [u8], Vec<u8>)> {
    let nul = body.iter().position(|&b| b == 0)?;
    let (keyword, rest) = body.split_at(nul);
    let rest = &rest[1..];
    let text = match kind {
        b"tEXt" => rest.to_vec(),
        b"zTXt" => inflate(rest.get(1..)?)?,
        b"iTXt" => {
            let (&compressed, rest) = rest.split_first()?;
            let rest = after_nul(rest.get(1..)?)?;
            let rest = after_nul(rest)?;
            if compressed == 1 {
                inflate(rest)?
            } else {
                rest.to_vec()
            }
        }
        _ => return None,
    };
    Some((keyword, text))
}

fn after_nul(bytes: &[u8]) -> Option<&[u8]> {
    let nul = bytes.iter().position(|&b| b == 0)?;
    bytes.get(nul + 1..)
}

fn inflate(bytes: &[u8]) -> Option<Vec<u8>> {
    read_capped(flate2::read::ZlibDecoder::new(bytes))
}

fn raw_profile(text: &[u8]) -> Option<Vec<u8>> {
    let start = text.iter().position(|b| !b.is_ascii_whitespace())?;
    let mut lines = text[start..].splitn(3, |&b| b == b'\n');
    lines.next()?;
    lines.next()?;
    let mut digits: Vec<u8> = lines
        .next()?
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .take_while(u8::is_ascii_hexdigit)
        .collect();
    digits.truncate(digits.len() & !1);
    hex::decode(digits).ok()
}
