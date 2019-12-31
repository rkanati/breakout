
pub mod rect;
pub mod linear;

pub use rect::Rect;
pub use linear::*;

use {
    crate::collider::Collider,
    ggez::nalgebra as na,
};

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct OrdF32(pub f32);

impl Eq for OrdF32 { }

impl Ord for OrdF32 {
    fn cmp(&self, rhs: &OrdF32) -> std::cmp::Ordering {
        self.partial_cmp(rhs)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub type V2 = na::Vector2<f32>;
pub type P2 = na::Point2<f32>;

pub fn left(v: V2) -> V2 {
    V2::new(-v.y, v.x)
}

pub fn right(v: V2) -> V2 {
    V2::new(v.y, -v.x)
}

pub fn reflect(v: V2, n: V2) -> V2 {
    v - 2. * n.dot(&v) * n
}

