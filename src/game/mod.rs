
mod pickups;
mod scoring;

pub use pickups::PickupKind;

use {
    self::{
        pickups::*,
        scoring::*,
    },
    crate::{
        block::{self, Block},
        collider::{Collider, Collision},
        dilate::Dilate,
        math::*,
    },
    rand::{
        distributions::{Bernoulli, Distribution},
        SeedableRng,
    },
    rand_core::RngCore,
    pcg_rand::Pcg32Basic,
};

const PADDLE_Y:         f32 = 40.;
const PADDLE_W:         f32 = 80.;
const PADDLE_MAX_SPEED: f32 = 600.;
const PADDLE_ACC:       f32 = 6000.;
const PADDLE_FRICTION:  f32 = 7.;

const BALL_SIZE:         f32 = 12.;
const BALL_SERVE_SPEED:  f32 = 400.;
const BALL_SERVE_COSINE: f32 = 0.7;

const GAME_WIDTH:  i32 = 600;
const GAME_HEIGHT: i32 = 600;

const GAME_LEFT:   i32 = -GAME_WIDTH / 2;
const GAME_RIGHT:  i32 =  GAME_WIDTH / 2;
const GAME_BOTTOM: i32 = 0;
const GAME_TOP:    i32 = GAME_HEIGHT;

const BLOCKS_VERT: i32 = 10;

const BLOCK_H: i32 = 30;
const SPLIT_STEP: i32 = BLOCK_H / 2;

fn serve_position(paddle_x: f32) -> P2 {
    P2::new(paddle_x, PADDLE_Y + 8.)
}

fn serve_velocity(paddle_vel: f32) -> V2 {
    let x_dir = if paddle_vel.abs () > 0.01 { paddle_vel.signum() }
                else                        { 0. };

    let dir = V2::new(BALL_SERVE_COSINE * x_dir, 1.).normalize();
    dir * BALL_SERVE_SPEED
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntityID {
    Walls,
    Paddle,
    Block(usize)
}

#[derive(Clone, Debug)]
struct SolidEntity {
    collider: Collider,
    id:       EntityID,
}

impl SolidEntity {
    fn new(collider: Collider, id: EntityID) -> SolidEntity {
        SolidEntity { collider, id }
    }
}

#[derive(Clone, Copy, Debug)]
struct Hit {
    collision: Collision,
    id:        EntityID,
}

struct FlyingBall {
    pos:            P2,
    prev_pos:       P2,
    vel:            V2,
    prev_collision: Option<(EntityID, V2)>,
}

enum Ball {
    Flying(FlyingBall),
    Serving
}

impl Ball {
    fn position(&self, alpha: f32) -> Option<P2> {
        if let Ball::Flying(ball) = self {
            let pos = ball.prev_pos.coords.lerp(&ball.pos.coords, alpha).into();
            Some(pos)
        }
        else {
            None
        }
    }

    fn serve(&mut self, pos: P2, vel: V2) {
        let ball = FlyingBall {
            pos,
            prev_pos: pos,
            vel,
            prev_collision: None,
        };
        *self = Ball::Flying(ball);
    }

    fn kill(&mut self) {
        *self = Ball::Serving;
    }
}

fn get_collision<'a> (solids: impl IntoIterator<Item = &'a SolidEntity>, motion: Segment)
    -> Option<Hit>
{
    solids.into_iter()
        .filter_map(|solid|
            solid.collider
                .intersect_with(motion)
                .map(|collision| Hit { collision, id: solid.id })
        )
        .min_by_key(|hit| OrdF32(hit.collision.param))
}

pub struct State {
    paddle_rect:   Rect,
    paddle_x:      f32,
    paddle_vel:    f32,
    paddle_prev_x: f32,

    ball_rect: Rect,
    ball:      Ball,

    blocks: Vec<Block>,
    pickups: Pickups,

    scoring: Scoring,

    solids: Vec<SolidEntity>,
}

#[derive(Clone)]
pub struct Frame<'a> {
    pub rect: Rect,

    pub paddle_rect: Rect,
    pub paddle_pos:  P2,

    pub ball_rect: Rect,
    pub ball_pos:  P2,

    pub blocks: &'a Vec<Block>,
    pub pickups: &'a Pickups,

    pub scoring: Scoring,
}

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub paddle_dir: i32,
    pub serve:      bool,
}

impl State {
    pub fn new(seed: u64) -> State {
        let paddle_rect = Rect::new(
            P2::new(-PADDLE_W * 0.5, -6.),
            P2::new( PADDLE_W * 0.5,  0.)
        );

        let ball_rect = Rect::new(
            P2::new(-BALL_SIZE * 0.5, -BALL_SIZE * 0.5),
            P2::new( BALL_SIZE * 0.5,  BALL_SIZE * 0.5)
        );

        let mut rand = Pcg32Basic::seed_from_u64(seed);
        let next_rand = Pcg32Basic::seed_from_u64(rand.next_u64());
        let mut split_distro = Bernoulli::new(0.3)
            .unwrap()
            .sample_iter(rand);

        const LAST_SPLIT: i32 = GAME_WIDTH / SPLIT_STEP;
        let splits: Vec<Vec<_>> = (1..BLOCKS_VERT)
            .map(|_| {
                let splits = (1 .. LAST_SPLIT)
                    .zip(split_distro.by_ref())
                    .filter(|(_, keep)| *keep)
                    .map(|(i, _)| i);
                let right_end = std::iter::once(LAST_SPLIT);
                splits.chain(right_end).collect()
            })
            .collect();

        let block_keep_distro = Bernoulli::new(0.95)
            .unwrap()
            .sample_iter(next_rand);

        let blocks = splits.iter()
            .enumerate()
            .flat_map(|(y_index, splits)| {
                let y0 = (GAME_TOP - (y_index + 2) as i32 * BLOCK_H) as f32;
                let y1 = y0 + BLOCK_H as f32;

                splits.iter()
                    .scan(0, move |l, r| {
                        assert!(*r > *l);

                        let x0 = GAME_LEFT as f32 + (*l * SPLIT_STEP) as f32;
                        let x1 = GAME_LEFT as f32 + (*r * SPLIT_STEP) as f32;
                        let rect = Rect::new(
                            P2::new(x0 + 0., y0 + 0.),
                            P2::new(x1 - 0., y1 - 0.)
                        );

                        let w = *r - *l;

                        use block::Kind::*;
                        let block = if w > 8 {
                            Block { kind: Invlunerable, rect: rect.contract(8.) }
                        }
                        else {
                            Block { kind: Scoring { score: w * 10, hp: w }, rect }
                        };

                        assert!(rect.width() > 0.);

                        *l = *r;
                        Some(block)
                    })
            })
            .zip(block_keep_distro)
            .filter(|(_, keep)| *keep)
            .map(|(block, _)| block)
            .collect();

        State {
            paddle_rect,
            paddle_x:      0.,
            paddle_prev_x: 0.,
            paddle_vel:    0.,

            ball_rect,
            ball: Ball::Serving,

            blocks,
            pickups: Pickups::new(seed),

            scoring: Scoring::new(),

            solids: Vec::new(),
        }
    }

    pub fn rect(&self) -> Rect {
        let mins = P2::new(GAME_LEFT  as f32, GAME_BOTTOM as f32);
        let dims = V2::new(GAME_WIDTH as f32, GAME_HEIGHT as f32);
        Rect::new_with_dims(mins, dims)
    }

    pub fn frame<'a> (&'a self, alpha: f32) -> Frame<'a> {
        let rect = self.rect();

        let paddle_x = lerp(self.paddle_prev_x, self.paddle_x, alpha);
        let paddle_pos = P2::new(paddle_x, PADDLE_Y);

        let ball_pos = self.ball.position(alpha)
            .unwrap_or(paddle_pos + V2::new(0., 10.));

        Frame {
            rect: self.rect(),

            paddle_rect: self.paddle_rect,
            paddle_pos,

            ball_rect: self.ball_rect,
            ball_pos,

            blocks: &self.blocks,
            pickups: &self.pickups,

            scoring: self.scoring,
        }
    }

    fn get_solids_for_entity(&mut self, entity: Rect) {
        self.solids.clear();

        use rect::CollideFrom;
        for (block_index, block) in self.blocks.iter().enumerate() {
            self.solids.push(SolidEntity::new(
                block.rect
                    .expand(entity)
                    .to_collider(CollideFrom::Outside),
                EntityID::Block(block_index)
            ));
        }

        let wall_collider = Rect::new(
                P2::new(GAME_LEFT  as f32, 0.),
                P2::new(GAME_RIGHT as f32, GAME_HEIGHT as f32)
            )
            .expand(entity)
            .to_collider(CollideFrom::Inside);
        self.solids.push(SolidEntity::new(wall_collider, EntityID::Walls));

        let paddle_collider = self.paddle_rect
            .at(P2::new(self.paddle_x, PADDLE_Y))
            .expand(entity)
            .side_max_y()
            .to_collider();
        self.solids.push(SolidEntity::new(paddle_collider, EntityID::Paddle));
    }

    fn update_ball(&mut self, dt: f32) {
        self.get_solids_for_entity(self.ball_rect);

        let ball = match &mut self.ball {
            Ball::Flying(ball) => ball,
            Ball::Serving      => { return; }
        };

        let mut remaining = dt;
        while remaining > 0. {
            ball.prev_pos = ball.pos;
            let motion = Segment::new(ball.pos, ball.vel * remaining);

            let Hit { collision, id } = match get_collision(&self.solids, motion) {
                Some(hit) => hit,
                None => {
                    ball.pos = motion.destination();
                    break;
                }
            };

            ball.vel = reflect(ball.vel, collision.normal);
            ball.pos = collision.point;

            use EntityID::*;
            match id {
                Walls if collision.normal.y > 0. => {
                    self.scoring.hit_floor();
                    self.ball.kill();
                    return;
                }

                Paddle => {
                    self.scoring.hit_paddle();

                    if self.paddle_vel.abs() > 5. {
                        ball.vel.x = ball.vel.x.abs()
                                   * self.paddle_vel.signum();
                    }
                }

                Block(index) => {
                    use block::Hit::*;
                    match self.blocks[index].hit() {
                        Broken(score) => {
                            let block = self.blocks.remove(index);
                            self.scoring.block_broken(score as i64);
                            self.pickups.block_broken(block);
                        }

                        Damaged => {
                            self.scoring.block_damaged();
                        }

                        Invlunerable => { }
                    }
                }

                _ => { }
            };

            remaining -= collision.param;
        }
    }

    pub fn update(&mut self, dt: f32, input: Input) -> bool {
        self.paddle_prev_x = self.paddle_x;

        //let friction = self.paddle_vel.signum() * (self.paddle_vel * self.paddle_vel) * 0.02;
        let friction = self.paddle_vel * PADDLE_FRICTION;
        let paddle_acc = input.paddle_dir as f32 * PADDLE_ACC - friction;

        self.paddle_vel = (self.paddle_vel + dt * paddle_acc)
            .min( PADDLE_MAX_SPEED)
            .max(-PADDLE_MAX_SPEED);

        let old_paddle_x = self.paddle_x;
        const PADDLE_X_POKEOUT: f32 = PADDLE_W * 0.5;
        const PADDLE_X_BOUND: f32 = (GAME_WIDTH as f32 - PADDLE_W + PADDLE_X_POKEOUT) * 0.5;
        self.paddle_x = (self.paddle_x + dt * self.paddle_vel)
            .min( PADDLE_X_BOUND)
            .max(-PADDLE_X_BOUND);
        self.paddle_vel = (self.paddle_x - old_paddle_x) / dt;

        if self.paddle_vel.abs() < 0.1 {
            self.paddle_vel = 0.;
        }

        let paddle_rect = self.paddle_rect.at(P2::new(self.paddle_x, PADDLE_Y));
        let collected = self.pickups.update(dt, paddle_rect, 0.);

        for pickup in collected {
            println!("got {:?}!", pickup);
        }

        match self.ball {
            Ball::Flying(_) => {
                self.update_ball(dt);
            }

            Ball::Serving if input.serve => {
                self.ball.serve(
                    serve_position(self.paddle_x),
                    serve_velocity(self.paddle_vel),
                );
            }

            _ => { }
        }

        let cleared = self.scoring.no_combo() && self.blocks.iter()
            .filter(|block| block.is_scoring())
            .count()
            == 0;

        if cleared {
            println!("A winner is you!");
            println!("Score:     {:8}",  self.scoring.score);
            println!("Penalties: {:8}",  self.scoring.penalties);
            println!("Rank:      {:>8}", self.scoring.rank());
            false
        }
        else {
            true
        }
    }
}
