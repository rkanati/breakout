
use {
    crate::{
        math::*,
    },
};

#[derive(Clone, Debug)]
pub struct Collider {
    edges: Vec<Segment>, // TODO use smallvec
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub param:  f32,
    pub point:  P2,
    pub normal: V2,
}

impl Collider {
    pub fn new(edges: Vec<Segment>) -> Collider {
        Collider { edges }
    }

    pub fn intersect_with(&self, line: impl Linear) -> Option<Collision> {
        self.edges.iter().copied()
            .map(|edge| (edge, right(edge.direction())))
            .filter(|(_, normal)| normal.dot(&line.stride()) < 0.)
            .filter_map(|(side, normal)| {
                line.intersect(&side)
                    .map(|ixn| Collision { param: ixn.lambda, point: ixn.point, normal })
            })
            .filter(|collision| collision.param > 0.00001)
            .min_by_key(|collision| OrdF32(collision.param))
    }
}

impl std::iter::FromIterator<Segment> for Collider {
    fn from_iter<T> (iter: T) -> Collider
        where T: IntoIterator<Item = Segment>
    {
        let edges = iter.into_iter().collect();
        Collider { edges }
    }
}

