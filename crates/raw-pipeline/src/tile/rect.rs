#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn full((w, h): (u32, u32)) -> Self {
        Self { x: 0, y: 0, w, h }
    }

    pub fn bounds(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        Self {
            x: x0,
            y: y0,
            w: x1.saturating_sub(x0),
            h: y1.saturating_sub(y0),
        }
    }

    pub fn dims(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> u32 {
        self.y + self.h
    }

    pub fn expand(self, margin: u32, frame: (u32, u32)) -> Self {
        Self::bounds(
            self.x.saturating_sub(margin),
            self.y.saturating_sub(margin),
            (self.right() + margin).min(frame.0),
            (self.bottom() + margin).min(frame.1),
        )
    }

    pub fn align(self, align: u32) -> Self {
        Self::bounds(
            self.x / align * align,
            self.y / align * align,
            self.right(),
            self.bottom(),
        )
    }

    pub fn union(self, other: Self) -> Self {
        Self::bounds(
            self.x.min(other.x),
            self.y.min(other.y),
            self.right().max(other.right()),
            self.bottom().max(other.bottom()),
        )
    }

    pub fn contains(self, inner: Self) -> bool {
        inner.x >= self.x
            && inner.y >= self.y
            && inner.right() <= self.right()
            && inner.bottom() <= self.bottom()
    }

    pub fn meets(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn scaled(self, factor: u32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            w: self.w * factor,
            h: self.h * factor,
        }
    }

    pub fn within(self, outer: Self) -> Self {
        Self {
            x: self.x - outer.x,
            y: self.y - outer.y,
            ..self
        }
    }

    pub fn rescaled(self, from: (u32, u32), to: (u32, u32)) -> Self {
        let floor = |v: u32, f: u32, t: u32| (u64::from(v) * u64::from(t) / u64::from(f)) as u32;
        let ceil = |v: u32, f: u32, t: u32| {
            (u64::from(v) * u64::from(t))
                .div_ceil(u64::from(f))
                .min(u64::from(t)) as u32
        };
        Self::bounds(
            floor(self.x, from.0, to.0),
            floor(self.y, from.1, to.1),
            ceil(self.right(), from.0, to.0),
            ceil(self.bottom(), from.1, to.1),
        )
    }

    pub fn grid((w, h): (u32, u32), edge: u32) -> impl Iterator<Item = Self> {
        let edge = edge.max(1);
        (0..h.div_ceil(edge)).flat_map(move |row| {
            (0..w.div_ceil(edge)).map(move |col| {
                let (x, y) = (col * edge, row * edge);
                Self::bounds(x, y, (x + edge).min(w), (y + edge).min(h))
            })
        })
    }
}
