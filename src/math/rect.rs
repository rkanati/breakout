
use {
    super::*,
    crate::{
        collider::Collider,
        dilate::Dilate,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollideFrom {
    Inside,
    Outside
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub mins: P2,
    pub maxs: P2
}

impl Rect {
    pub fn new_unchecked(mins: P2, maxs: P2) -> Rect {
        debug_assert!(mins.x <= maxs.x);
        debug_assert!(mins.y <= maxs.y);
        Rect { mins, maxs }
    }

    pub fn new(a: P2, b: P2) -> Rect {
        let minx = a.x.min(b.x);
        let miny = a.y.min(b.y);
        let maxx = a.x.max(b.x);
        let maxy = a.y.max(b.y);
        Rect::new_unchecked(P2::new(minx, miny), P2::new(maxx, maxy))
    }

    pub fn new_with_dims(a: P2, dims: V2) -> Rect {
        Self::new(a, a + dims)
    }

    pub fn at(&self, origin: P2) -> Rect {
        let v = origin.coords;
        Rect { mins: self.mins + v, maxs: self.maxs + v }
    }

    pub fn width(&self) -> f32 {
        self.maxs.x - self.mins.x
    }

    pub fn height(&self) -> f32 {
        self.maxs.y - self.mins.y
    }

    pub fn dims(&self) -> V2 {
        V2::new(self.width(), self.height())
    }

    pub fn contains(&self, p: P2) -> bool {
           (self.mins.x .. self.maxs.x).contains(&p.x)
        && (self.mins.y .. self.maxs.y).contains(&p.y)
    }

    pub fn vertices(&self) -> [P2; 4] {
        [   self.mins,
            P2::new(self.maxs.x, self.mins.y),
            self.maxs,
            P2::new(self.mins.x, self.maxs.y)
        ]
    }

    pub fn side_min_x(&self) -> Segment {
        Segment::new_from_points(P2::new(self.mins.x, self.maxs.y), self.mins)
    }

    pub fn side_max_x(&self) -> Segment {
        Segment::new_from_points(P2::new(self.maxs.x, self.mins.y), self.maxs)
    }

    pub fn side_min_y(&self) -> Segment {
        Segment::new_from_points(self.mins, P2::new(self.maxs.x, self.mins.y))
    }

    pub fn side_max_y(&self) -> Segment {
        Segment::new_from_points(self.maxs, P2::new(self.mins.x, self.maxs.y))
    }

    pub fn sides(&self) -> [Segment; 4] {
        [   self.side_min_y(),
            self.side_max_x(),
            self.side_max_y(),
            self.side_min_x()
        ]
    }

    pub fn to_collider(self, from: CollideFrom) -> Collider {
        self.sides().iter().copied()
            .map(|edge| match from {
                CollideFrom::Inside  => edge.reverse(),
                CollideFrom::Outside => edge
            })
            .collect()
    }
}

impl Dilate<Rect> for Rect {
    type Output = Rect;

    fn expand(&self, by: Rect) -> Rect {
        Rect::new_unchecked(self.mins + by.mins.coords, self.maxs + by.maxs.coords)
    }

    fn contract(&self, by: Rect) -> Rect {
        Rect::new_unchecked(self.mins + by.maxs.coords, self.maxs + by.mins.coords)
    }
}

impl Dilate<f32> for Rect {
    type Output = Rect;

    fn expand(&self, by: f32) -> Rect {
        let by = V2::new(by, by);
        Rect::new_unchecked(self.mins - by, self.maxs + by)
    }

    fn contract(&self, by: f32) -> Rect {
        let by = V2::new(by, by);
        Rect::new_unchecked(self.mins + by, self.maxs - by)
    }
}

impl From<Rect> for ggez::graphics::Rect {
    fn from(rect: Rect) -> Self {
        ggez::graphics::Rect::new(rect.mins.x, rect.mins.y, rect.width(), rect.height())
    }
}
