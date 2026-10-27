use super::opcodes::{BadPixelList, OpcodeList};
use crate::decode::mosaic::ActiveArea;
use rawler::RawImage;
use rawler::rawimage::RawImageData;

const MAX_FLAGGED: usize = 1 << 20;

pub(super) fn flagged_pixels(
    constant: Option<u32>,
    lists: &[(OpcodeList, BadPixelList)],
    raw: &RawImage,
) -> Vec<[usize; 2]> {
    let area = ActiveArea::of(raw);
    let constant = constant
        .map(|value| constant_sites(value, raw, area))
        .unwrap_or_default();
    let listed = lists.iter().flat_map(|(list, bad)| {
        let origin = match list {
            OpcodeList::One => area.origin,
            OpcodeList::Two | OpcodeList::Three => [0, 0],
        };
        listed_sites(bad, origin, area)
    });
    let mut flagged: Vec<[usize; 2]> = constant
        .into_iter()
        .chain(listed)
        .take(MAX_FLAGGED + 1)
        .collect();
    if flagged.len() > MAX_FLAGGED {
        tracing::warn!("truncating DNG bad pixel list at {MAX_FLAGGED} sites");
        flagged.truncate(MAX_FLAGGED);
    }
    flagged
}

fn listed_sites(
    bad: &BadPixelList,
    origin: [usize; 2],
    area: ActiveArea,
) -> impl Iterator<Item = [usize; 2]> + '_ {
    let points = bad
        .points
        .iter()
        .filter_map(move |&[row, col]| local(area, [col, row], origin));
    let rects = bad
        .rects
        .iter()
        .flat_map(move |&rect| clip(area, rect, origin));
    points.chain(rects)
}

fn local(area: ActiveArea, [x, y]: [u32; 2], origin: [usize; 2]) -> Option<[usize; 2]> {
    let x = (x as usize).checked_sub(origin[0])?;
    let y = (y as usize).checked_sub(origin[1])?;
    (x < area.size[0] && y < area.size[1]).then_some([x, y])
}

fn clip(
    area: ActiveArea,
    [top, left, bottom, right]: [u32; 4],
    origin: [usize; 2],
) -> impl Iterator<Item = [usize; 2]> {
    let span = |lo: u32, hi: u32, axis: usize| {
        let lo = (lo as usize).saturating_sub(origin[axis]);
        let hi = (hi as usize)
            .saturating_sub(origin[axis])
            .min(area.size[axis]);
        lo..hi.max(lo)
    };
    let rows = span(top, bottom, 1);
    let cols = span(left, right, 0);
    rows.flat_map(move |y| cols.clone().map(move |x| [x, y]))
}

fn constant_sites(value: u32, raw: &RawImage, area: ActiveArea) -> Vec<[usize; 2]> {
    let RawImageData::Integer(src) = &raw.data else {
        return Vec::new();
    };
    let [x0, y0] = area.origin;
    let [width, height] = area.size;
    let stride = raw.width * raw.cpp;
    let fits = raw.cpp == 1 && area.fits(stride, src.len() / stride.max(1));
    if !fits {
        return Vec::new();
    }
    (0..height)
        .flat_map(|y| {
            let start = (y0 + y) * stride + x0;
            src[start..start + width]
                .iter()
                .enumerate()
                .filter(move |(_, v)| u32::from(**v) == value)
                .map(move |(x, _)| [x, y])
        })
        .take(MAX_FLAGGED + 1)
        .collect()
}
