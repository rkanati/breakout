
pub trait Convex {
    fn furthest_along(&self, direction: V2) -> P2;
}

impl Convex for Rect {
    fn furthest_along(&self, direction: V2) -> P2 {
        let x = if direction.x < 0. { self.mins.x }
                else                { self.maxs.x };
        let y = if direction.y < 0. { self.mins.y }
                else                { self.maxs.y };
        P2::new(x, y)
    }
}

fn gjk_relative(path: Segment, moving: impl Convex, fixed: impl Convex) {
}

fn gjk(
    path_a: Segment, convex_a: impl Convex,
    path_b: Segment, convex_b: impl Convex)
{
    gjk_relative(path_a.relative_to(path_b), convex_a, convex_b)
}

