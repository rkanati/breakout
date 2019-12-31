
use super::*;

pub struct Intersection {
    pub lambda: f32,
    pub mu:     f32,
    pub point:  P2,
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

    fn intersect(&self, other: &impl Linear) -> Option<Intersection> {
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
                let point = la.at(lambda);
                Some(Intersection { lambda, mu, point })
            }
            else {
                None
            }
        }
    }

    fn relative_to(&self, other: impl Linear) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_intersetion() {
        let la = Line::new(P2::new(0., 0.), V2::new(  6., 6.));
        let lb = Line::new(P2::new(9., 0.), V2::new(-12., 6.));
        let Intersection { lambda, mu, point } = la.intersect(&lb).unwrap();
        eprintln!("lambda={}, mu={}, {:?}", lambda, mu, point);
        assert!((lambda - 0.5).abs () < 0.00001);
        assert!((mu     - 0.5).abs () < 0.00001);
        assert!((point - P2::new(3., 3.)).norm() < 0.00001);
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
    fn relative_to(&self, other: impl Linear) -> Line {
        let other_line = other.whole_line();
        let source = self.source - other_line.source;
        let stride = self.stride - other_line.stride;
        Line::new(source.into(), stride)
    }
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
    fn relative_to(&self, other: impl Linear) -> Ray {
        Ray(self.whole_line().relative_to(other))
    }
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

    pub fn reverse(self) -> Segment {
        let source = self.destination();
        let stride = -self.0.stride;
        Segment::new(source, stride)
    }

    pub fn to_collider(self) -> Collider {
        Collider::new(vec![self])
    }
}

impl Linear for Segment {
    fn whole_line(&self) -> Line { self.0 }
    fn parameter_on(&self, t: f32) -> bool { t >= 0. && t <= 1. }
    fn relative_to(&self, other: impl Linear) -> Segment {
        Segment(self.whole_line().relative_to(other))
    }
}

