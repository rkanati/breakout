
use {
    crate::{
        block::Block,
        dilate::Dilate,
        math::*,
    },
    rand::{Rng, SeedableRng},
    pcg_rand::Pcg32Basic,
};

const DROP_CHANCE: f32 = 0.5;
const DROP_SPEED:  f32 = 300.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    ExtraBall,
    Detonator,
}

#[derive(Clone, Copy, Debug)]
pub struct Pickup {
    pub position: P2,
    pub kind:     PickupKind,
}

pub struct Pickups {
    rng:     Pcg32Basic,
    pickups: Vec<Pickup>,
}

impl<'a> IntoIterator for &'a Pickups {
    type Item = &'a Pickup;
    type IntoIter = std::slice::Iter<'a, Pickup>;
    fn into_iter(self) -> Self::IntoIter {
        self.pickups.iter()
    }
}

impl Pickups {
    pub fn new(seed: u64) -> Pickups {
        let rng = Pcg32Basic::seed_from_u64(seed);
        let pickups = Vec::new();

        Pickups { rng, pickups }
    }

    pub fn block_broken(&mut self, block: Block) {
        if self.rng.gen::<f32>() >= DROP_CHANCE {
            return;
        }

        use PickupKind::*;
        let kind = match self.rng.gen::<f32>() {
            x if x < 0.5 => ExtraBall,
            _            => Detonator,
        };

        let position = block.rect.mins + 0.5 * block.rect.dims();

        let pickup = Pickup { position, kind };
        self.pickups.push(pickup);
    }

    pub fn update(&mut self, dt: f32, paddle_rect: Rect, floor_level: f32) -> Vec<PickupKind> {
        let drop_rect = Rect::new(P2::new(-8., -8.), P2::new(8., 8.));
        let paddle_rect = paddle_rect.expand(drop_rect);

        for pickup in self.pickups.iter_mut() {
            pickup.position.y -= dt * DROP_SPEED;
        }

        let mut collected = Vec::new();
        self.pickups.retain(|pickup| {
            let hit = paddle_rect.contains(pickup.position);
            if hit { collected.push(pickup.kind); }
            !hit && pickup.position.y > 0.
        });

        collected
    }
}

