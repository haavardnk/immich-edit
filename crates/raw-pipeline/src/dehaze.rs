#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DehazeGrid {
    pub scale: u32,
    pub patch: u32,
    pub guided: u32,
}

impl DehazeGrid {
    pub fn for_dims((w, h): (u32, u32)) -> Self {
        let min_dim = w.min(h);
        let half_min = (min_dim / 2).max(1);
        let patch_full = (min_dim / 200).max(8).min(half_min);
        let guided_full = (min_dim / 50).max(16).min(half_min);
        let scale = if min_dim >= 512 { 4 } else { 1 };
        Self {
            scale,
            patch: (patch_full / scale).max(2),
            guided: (guided_full / scale).max(4),
        }
    }

    pub fn support(&self) -> u32 {
        self.scale * (self.patch + 2 * self.guided + 2)
    }
}
