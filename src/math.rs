
use {
    ggez::nalgebra as na,
};

pub type V2 = na::Vector2<f32>;
pub type P2 = na::Point2<f32>;

fn left(v: V2) -> V2 {
    V2::new(-v.y, v.x)
}

fn right(v: V2) -> V2 {
    V2::new(v.y, -v.x)
}

pub trait Linear {
    fn whole_line(&self) -> Line;
    fn parameter_on(&self, lambda: f32) -> bool;
    fn stride(&self) -> V2 {
        self.whole_line().stride
    }
    fn direction(&self) -> V2 {
        self.stride().normalize()
    }
    fn project(&self, p: P2) -> f32 {
        let line = self.whole_line();
        let v = p - line.source;
        v.dot(&line.stride) // TODO: unit?
    }
    fn intersect(&self, other: &impl Linear) -> Option<(f32, f32, P2)> {
        let la = self.whole_line();
        let lb = other.whole_line();

        let denom = lb.stride.y * la.stride.x - lb.stride.x * la.stride.y;
        if denom.abs () < 0.00001 {
            None
        }
        else {
            let offset = lb.source - la.source;
            let lambda = (lb.stride.y * offset.x - lb.stride.x * offset.y) / denom;
            let mu     = (la.stride.y * offset.x - la.stride.x * offset.y) / denom;

            if self.parameter_on(lambda) && other.parameter_on(mu) {
                let p = la.at(lambda);
                Some((lambda, mu, p))
            }
            else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_intersetion() {
        let la = Line::new(P2::new(0., 0.), V2::new(  6., 6.));
        let lb = Line::new(P2::new(9., 0.), V2::new(-12., 6.));
        let (t, u, p) = la.intersect(&lb).unwrap();
        eprintln!("t={}, u={}, {:?}", t, u, p);
        assert!((t - 0.5).abs () < 0.00001);
        assert!((t - 0.5).abs () < 0.00001);
        assert!((p - P2::new(3., 3.)).norm() < 0.00001);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Line {
    source: P2,
    stride: V2,
}

impl Line {
    pub fn new(source: P2, stride: V2) -> Line {
        Line { source, stride }
    }

    pub fn at(&self, t: f32) -> P2 {
        self.source + t * self.stride
    }
}

impl Linear for Line {
    fn whole_line(&self) -> Line { *self }
    fn parameter_on(&self, _: f32) -> bool { true }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray(Line);

impl Ray {
    pub fn new(source: P2, stride: V2) -> Ray {
        Ray(Line::new(source, stride))
    }
}

impl Linear for Ray {
    fn whole_line(&self) -> Line { self.0 }
    fn parameter_on(&self, t: f32) -> bool { t >= 0. }
}

#[derive(Clone, Copy, Debug)]
pub struct Segment(Line);

impl Segment {
    pub fn new(source: P2, stride: V2) -> Segment {
        Segment(Line::new(source, stride))
    }

    pub fn new_from_points(a: P2, b: P2) -> Segment {
        Self::new(a, b - a)
    }

    pub fn source(&self) -> P2 {
        self.0.source
    }

    pub fn destination(&self) -> P2 {
        self.source() + self.0.stride
    }
}

impl Linear for Segment {
    fn whole_line(&self) -> Line { self.0 }
    fn parameter_on(&self, t: f32) -> bool { t >= 0. && t <= 1. }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    mins: P2,
    maxs: P2
}

#[derive(Clone, Copy, Debug)]
pub enum IntersectFrom {
    Inside,
    Outside
}

impl Rect {
    pub fn new(a: P2, b: P2) -> Rect {
        let minx = a.x.min(b.x);
        let miny = a.y.min(b.y);
        let maxx = a.x.max(b.x);
        let maxy = a.y.max(b.y);
        Rect { mins: P2::new(minx, miny), maxs: P2::new(maxx, maxy) }
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

    pub fn vertices(&self) -> [P2; 4] {
        [   self.mins,
            P2::new(self.maxs.x, self.mins.y),
            self.maxs,
            P2::new(self.mins.x, self.maxs.y)
        ]
    }

    pub fn sides(&self) -> [Segment; 4] {
        let vs = self.vertices();
        [   Segment::new_from_points(vs[0], vs[1]),
            Segment::new_from_points(vs[1], vs[2]),
            Segment::new_from_points(vs[2], vs[3]),
            Segment::new_from_points(vs[3], vs[0]),
        ]
    }

    pub fn intersect_with(&self, from: IntersectFrom, line: impl Linear) -> Option<(f32, P2, V2)> {
        let sides = self.sides();
        let normals = sides.iter()
            .map(|side| {
                let along = side.direction();
                match from {
                    IntersectFrom::Outside => right(along),
                    IntersectFrom::Inside  => left(along),
                }
            });

        sides.iter()
            .zip(normals)
            .filter(|(_, normal)| normal.dot(&line.stride()) < 0.)
            .filter_map(|(side, normal)| {
                let ix = line.intersect(side)?;
                Some((ix, normal))
            })
            .filter(|(ix, _)| ix.0 > 0.00001)
            .min_by(|(ia, _), (ib, _)| {
                ia.0.partial_cmp(&ib.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(ix, normal)| (ix.0, ix.2, normal))
    }
}

impl From<Rect> for ggez::graphics::Rect {
    fn from(rect: Rect) -> Self {
        ggez::graphics::Rect::new(rect.mins.x, rect.mins.y, rect.width(), rect.height())
    }
}

