
use {
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

mod scoring {
    #[derive(Clone, Copy, Debug)]
    pub struct Scoring {
        pub score:            i64,
        pub combo_score:      i64,
        pub combo_multiplier: f64,
        pub combo_max:        i64,
        pub penalties:        i64,
    }

    impl Scoring {
        pub fn new() -> Scoring {
            Scoring {
                score:            0,
                combo_score:      0,
                combo_multiplier: 1.,
                combo_max:        0,
                penalties:        0,
            }
        }

        fn end_combo(&mut self) -> i64 {
            let combo = self.combo_score;
            self.combo_score = 0;
            self.combo_max = self.combo_max.max(combo);
            self.combo_multiplier = 1.;
            combo
        }

        pub fn hit_floor(&mut self) {
            let combo = self.end_combo();
            self.score     -= combo;
            self.penalties += combo;
        }

        pub fn hit_paddle(&mut self) {
            let combo = self.end_combo();
            self.score += combo;
        }

        pub fn block_broken(&mut self, block_score: i64) {
            let score = self.combo_multiplier * block_score as f64;
            self.combo_score += score.round() as i64;
            self.combo_multiplier += 1.;
        }

        pub fn block_damaged(&mut self) {
            self.combo_multiplier += 0.1;
        }

        pub fn no_combo(&self) -> bool {
            self.combo_score == 0
        }

        pub fn rank(&self) -> Rank {
            match ((self.penalties as f64 / self.score as f64) * 1000.).trunc() as i64 {
                  0 ..=   5 => Rank::S,
                  6 ..=  25 => Rank::A,
                 26 ..=  50 => Rank::B,
                 51 ..= 100 => Rank::C,
                101 ..= 150 => Rank::D,
                151 ..= 250 => Rank::E,
                _           => Rank::F
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Rank {
        F = 0,
        E = 1,
        D = 2,
        C = 3,
        B = 4,
        A = 5,
        S = 6,
    }

    impl std::fmt::Display for Rank {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            let string = match *self {
                Rank::F => "F",
                Rank::E => "E",
                Rank::D => "D",
                Rank::C => "C",
                Rank::B => "B",
                Rank::A => "A",
                Rank::S => "S",
            };
            f.write_str(string)
        }
    }
}

use self::scoring::*;

pub struct State {
    paddle_rect:   Rect,
    paddle_x:      f32,
    paddle_vel:    f32,
    paddle_prev_x: f32,

    ball_rect:           Rect,
    ball_pos:            P2,
    ball_prev_pos:       P2,
    ball_vel:            V2,
    ball_prev_collision: Option<(EntityID, V2)>,

    blocks: Vec<Block>,

    scoring: Scoring,
}

#[derive(Clone)]
pub struct Frame<'a> {
    pub paddle_rect: Rect,
    pub paddle_pos:  P2,

    pub ball_rect: Rect,
    pub ball_pos:  P2,

    pub blocks: &'a Vec<Block>,

    pub scoring: Scoring,
}

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub paddle_dir: i32,
}

impl State {
    pub fn width(&self) -> i32 {
        GAME_WIDTH
    }

    pub fn height(&self) -> i32 {
        GAME_HEIGHT
    }

    pub fn left(&self) -> i32 {
        GAME_LEFT
    }

    pub fn top(&self) -> i32 {
        GAME_TOP
    }

    pub fn frame<'a> (&'a self, alpha: f32) -> Frame<'a> {
        let paddle_x = lerp(self.paddle_prev_x, self.paddle_x, alpha);
        let paddle_pos = P2::new(paddle_x, PADDLE_Y);

        let ball_pos = P2::from(
            self.ball_prev_pos.coords.lerp(&self.ball_pos.coords, alpha)
        );

        Frame {
            paddle_rect: self.paddle_rect,
            paddle_pos,

            ball_rect: self.ball_rect,
            ball_pos,

            blocks: &self.blocks,

            scoring: self.scoring,
        }
    }

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

        let ball_pos = P2::new(0., PADDLE_Y + 10.);
        let ball_vel = V2::new(BALL_SERVE_COSINE, 1.0).normalize() * BALL_SERVE_SPEED;

        State {
            paddle_rect,
            paddle_x:      0.,
            paddle_prev_x: 0.,
            paddle_vel:    0.,

            ball_rect,
            ball_pos,
            ball_prev_pos: ball_pos,
            ball_vel,
            ball_prev_collision: None,

            blocks,

            scoring: Scoring::new(),
        }
    }

    fn get_solids(&self, solids: &mut Vec<SolidEntity>) {
        solids.clear();

        use rect::CollideFrom;
        for (block_index, block) in self.blocks.iter().enumerate() {
            solids.push(SolidEntity::new(
                block.rect.to_collider(CollideFrom::Outside),
                EntityID::Block(block_index)
            ));
        }

        let wall_collider = Rect::new(
            P2::new(GAME_LEFT  as f32, 0.),
            P2::new(GAME_RIGHT as f32, GAME_HEIGHT as f32)
        ).to_collider(CollideFrom::Inside);
        solids.push(SolidEntity::new(wall_collider, EntityID::Walls));

        let paddle_collider = self.paddle_rect
            .at(P2::new(self.paddle_x, PADDLE_Y))
            .side_max_y()
            .to_collider();
        solids.push(SolidEntity::new(paddle_collider, EntityID::Paddle));
    }

    fn get_ball_collision(&self, solids: &mut Vec<SolidEntity>, motion: Segment)
        -> Option<Hit>
    {
        self.get_solids(solids);

        solids.iter()
            .filter_map(|solid|
                solid.collider
                    .intersect_with(motion)
                    .map(|collision| Hit { collision, id: solid.id })
            )
            .min_by_key(|hit| OrdF32(hit.collision.param))
    }

    fn handle_collision(&mut self, ball_motion: Segment) -> Option<(P2, f32)> {
        let mut colliders = Vec::new();

        let Hit { collision, id } = match self.get_ball_collision(&mut colliders, ball_motion) {
            Some(hit) => hit,
            None      => { return None; }
        };

        let new_collision = Some((id, collision.normal));
        if self.ball_prev_collision == new_collision {
            return None;
        }

        use EntityID::*;
        match id {
            Walls => {
                if collision.normal.y > 0. {
                    println!("oh no! you lose {} points!", self.scoring.combo_score);
                    self.scoring.hit_floor();
                }

                self.ball_vel = reflect(self.ball_vel, collision.normal);
            }

            Paddle => {
                self.scoring.hit_paddle();

                self.ball_vel = reflect(self.ball_vel, collision.normal);
                if self.paddle_vel.abs() > 4. {
                    self.ball_vel.x = self.ball_vel.x.abs()
                                    * self.paddle_vel.signum();
                }
            }

            Block(index) => {
                use block::Hit::*;
                match self.blocks[index].hit() {
                    Broken(score) => {
                        self.scoring.block_broken(score as i64);
                        self.blocks.remove(index);
                    }

                    Damaged => {
                        self.scoring.block_damaged();
                    }

                    Invlunerable => { }
                }

                self.ball_vel = reflect(self.ball_vel, collision.normal);
            }
        };

        self.ball_prev_collision = new_collision;
        Some((collision.point, collision.param))
    }

    pub fn update(&mut self, dt: f32, input: Input) -> bool {
        self.paddle_prev_x = self.paddle_x;
        self.ball_prev_pos = self.ball_pos;

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

        let mut remaining = dt;
        while remaining > 0. {
            let ball_motion = Segment::new(
                self.ball_pos,
                self.ball_vel * remaining
            );

            match self.handle_collision(ball_motion) {
                Some((point, travelled)) => {
                    self.ball_pos = point;
                    remaining -= travelled;
                },
                None => {
                    self.ball_pos = ball_motion.destination();
                    break;
                }
            }
        }

        let cleared = self.scoring.no_combo() && self.blocks.iter()
            .filter(|block| block.is_scoring())
            .count()
            == 0;

        if cleared {
            println!("A winner is you!");
            println!("Score:     {:8}", self.scoring.score);
            println!("Penalties: {:8}", self.scoring.penalties);
            println!("Rank:      {}",   self.scoring.rank());
            false
        }
        else {
            true
        }
    }
}
