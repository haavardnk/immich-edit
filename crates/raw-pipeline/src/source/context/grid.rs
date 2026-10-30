pub(crate) const BILINEAR_SLACK: u32 = 2;
const CROP_SLACK: u32 = 2 * BILINEAR_SLACK + 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextPart {
    Haze,
    Texture,
    Guide,
    Shadows,
}

impl ContextPart {
    pub const ALL: [Self; 4] = [Self::Haze, Self::Texture, Self::Guide, Self::Shadows];

    pub fn channels(self) -> usize {
        match self {
            Self::Haze | Self::Guide => 2,
            Self::Texture | Self::Shadows => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextGrid {
    pub level: u32,
    pub origin: (u32, u32),
    pub dims: (u32, u32),
}

impl ContextGrid {
    pub(crate) fn fits(&self, full: (u32, u32), window: (u32, u32)) -> bool {
        let inside = |origin: u32, size: u32, full: u32, window: u32| {
            let frame = (full >> self.level).max(1);
            size > 0
                && size <= (window >> self.level) + CROP_SLACK
                && origin.checked_add(size).is_some_and(|end| end <= frame)
        };
        self.level < u32::BITS
            && inside(self.origin.0, self.dims.0, full.0, window.0)
            && inside(self.origin.1, self.dims.1, full.1, window.1)
    }

    pub(crate) fn texels(&self) -> u64 {
        u64::from(self.dims.0) * u64::from(self.dims.1)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceContext {
    grids: [Option<ContextGrid>; 4],
}

impl SourceContext {
    pub fn get(&self, part: ContextPart) -> Option<ContextGrid> {
        self.grids[part as usize]
    }

    pub fn set(&mut self, part: ContextPart, grid: ContextGrid) {
        self.grids[part as usize] = Some(grid);
    }

    pub fn grids(&self) -> impl Iterator<Item = (ContextPart, ContextGrid)> + '_ {
        ContextPart::ALL
            .into_iter()
            .filter_map(|part| self.get(part).map(|grid| (part, grid)))
    }

    pub fn samples(&self) -> u64 {
        self.grids().fold(0, |sum, (part, grid)| {
            sum.saturating_add(grid.texels().saturating_mul(part.channels() as u64))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grids_must_lie_inside_their_level() {
        let grid = |level, origin, dims| ContextGrid {
            level,
            origin,
            dims,
        };
        let full = (1203, 805);
        for (label, grid, window, fits) in [
            ("whole level", grid(2, (0, 0), (300, 201)), full, true),
            ("past the edge", grid(2, (1, 0), (300, 201)), full, false),
            ("empty", grid(3, (4, 4), (0, 5)), full, false),
            ("past the top level", grid(40, (0, 0), (1, 1)), full, false),
            (
                "one texel at the top",
                grid(11, (0, 0), (1, 1)),
                (1, 1),
                true,
            ),
            (
                "wider than its window",
                grid(2, (0, 0), (40, 20)),
                (120, 80),
                false,
            ),
            (
                "the window plus slack",
                grid(2, (0, 0), (36, 26)),
                (120, 80),
                true,
            ),
        ] {
            if grid.fits(full, window) != fits {
                panic!("{label}: fits is {}", !fits);
            }
        }
    }
}
