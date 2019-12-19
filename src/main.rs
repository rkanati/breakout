
mod math;

use {
    crate::math::*,
    ggez::{self, event, graphics, nalgebra as na, input::keyboard, Context, GameResult},
    rand::distributions::Uniform,
    pcg_rand::Pcg32Basic,
};

const PADDLE_Y: f32 = 80.;
const PADDLE_W: f32 = 100.;
const PADDLE_SPEED: f32 = 8.;

const BALL_SERVE_SPEED: f32 = 8.;
const BALL_MAX_X_SPEED: f32 = 16.;

const KEY_LEFT : keyboard::KeyCode = keyboard::KeyCode::A;
const KEY_RIGHT: keyboard::KeyCode = keyboard::KeyCode::D;

const GAME_WIDTH:  f32 = 500.;
const GAME_HEIGHT: f32 = 800.;

const BLOCKS_HORZ: i32 = 10;
const BLOCKS_VERT: i32 = 16;

const BLOCK_W: f32 = GAME_WIDTH / BLOCKS_HORZ as f32;
const BLOCK_H: f32 = 20.;

struct Block {
    pub hp:  i32,
    pub pos: P2,
}

struct State {
    paddle_mesh: graphics::Mesh,
    ball_mesh:   graphics::Mesh,

    paddle_rect: Rect,
    block_rect:  Rect,

    paddle_x: f32,

    ball_pos: P2,
    ball_vel: V2,
    ball_prev_collision: Option<(Collider, V2)>,

    blocks: Vec<Block>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Collider {
    Walls,
    Paddle,
    Block(usize)
}

#[derive(Clone, Copy, Debug)]
struct Hit {
    param:    f32,
    point:    P2,
    normal:   V2,
    collider: Collider,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let paddle_rect = Rect::new(
            P2::new(-PADDLE_W * 0.5, -6.),
            P2::new( PADDLE_W * 0.5,  0.)
        );

        let paddle_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            paddle_rect.into(),
            graphics::WHITE,
        )?;

        let ball_rect = graphics::Rect::new(-3., -3., 6., 6.);

        let ball_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            ball_rect,
            graphics::Color::new(1., 0., 0., 1.),
        )?;

        let rand = Pcg32Basic::seed_from_u64(12345);

        let blocks = (0..BLOCKS_VERT)
            .flat_map(|y| {
                const SPLIT_STEP: u32 = 20;
                let split_xs = Uniform::new(0, GAME_WIDTH / SPLIT_STEP as f32)
                    .sample_iter();

                let splits =  
            })

            .map(|(x, y)| Block {
                hp: (3 - (y/2) % 3),
                pos: P2::new(
                    x as f32 * BLOCK_W - GAME_WIDTH * 0.5,
                    y as f32 * BLOCK_H + (GAME_HEIGHT - BLOCK_H * BLOCKS_VERT as f32)
                )
            })
            .collect();

        let block_rect = Rect::new(P2::new(1., 1.), P2::new(BLOCK_W-2., BLOCK_H-2.));

        Ok(State {
            paddle_mesh,
            ball_mesh,

            paddle_rect,
            block_rect,

            paddle_x: 0.,

            ball_pos: P2::new(0., PADDLE_Y + 10.),
            ball_vel: V2::new(1., 1.).normalize() * BALL_SERVE_SPEED,
            ball_prev_collision: None,

            blocks
        })
    }

    fn get_colliders(&self) -> Vec<(Rect, IntersectFrom, Collider)> {
        let mut colliders = Vec::new();

        for (block_index, block) in self.blocks.iter().enumerate() {
            let rect = self.block_rect.at(block.pos);
            colliders.push((rect, IntersectFrom::Outside, Collider::Block(block_index)));
        }

        let wall_rect = Rect::new(
            P2::new(-GAME_WIDTH*0.5, 0.),
            P2::new( GAME_WIDTH*0.5, GAME_HEIGHT)
        );
        colliders.push((wall_rect, IntersectFrom::Inside, Collider::Walls));

        let paddle_rect = self.paddle_rect.at(P2::new(self.paddle_x, PADDLE_Y));
        colliders.push((paddle_rect, IntersectFrom::Outside, Collider::Paddle));

        colliders
    }

    fn get_collision(&self, motion: Segment) -> Option<Hit> {
        let colliders = self.get_colliders();

        let mut nearest_hit: Option<Hit> = None;

        for (rect, from, collider) in colliders.iter() {
            if let Some((param, point, normal)) = rect.intersect_with(*from, motion) {
                if param < nearest_hit.map_or(std::f32::INFINITY, |h| h.param) {
                    nearest_hit = Some(Hit { param, point, normal, collider: *collider });
                }
            }
        }

        nearest_hit
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let left  = keyboard::is_key_pressed(ctx, KEY_LEFT);
        let right = keyboard::is_key_pressed(ctx, KEY_RIGHT);

        let paddle_dir =
            if      left && !right { -1 }
            else if right && !left {  1 }
            else                   {  0 };

        const PADDLE_X_BOUND: f32 = (GAME_WIDTH - PADDLE_W) * 0.5;
        self.paddle_x = (self.paddle_x + paddle_dir as f32 * PADDLE_SPEED)
            .min( PADDLE_X_BOUND)
            .max(-PADDLE_X_BOUND);

        let mut remaining = 1.;

        while remaining > 0.00001 {
            let ball_motion = Segment::new(
                self.ball_pos,
                self.ball_vel * remaining
            );

            let (point, travel) = {
                let hit = self.get_collision(ball_motion);

                if let Some(hit) = hit {
                    let collision = Some((hit.collider, hit.normal));
                    if self.ball_prev_collision != collision {
                        use Collider::*;
                        match hit.collider {
                            Walls => {
                                self.ball_vel -= 2. * hit.normal.dot(&self.ball_vel) * hit.normal;
                            }
                            Paddle => {
                                self.ball_vel -= 2. * hit.normal.dot(&self.ball_vel) * hit.normal;
                                if paddle_dir != 0 {
                                    self.ball_vel.x = self.ball_vel.x.abs() * paddle_dir as f32;
                                }
                            }
                            Block(index) => {
                                self.blocks[index].hp -= 1;

                                if self.blocks[index].hp <= 0 {
                                    self.blocks.remove(index);
                                }

                                self.ball_vel -= 2. * hit.normal.dot(&self.ball_vel) * hit.normal;
                            }
                        };
                        self.ball_prev_collision = collision;
                        (hit.point, hit.param)
                    }
                    else {
                        (ball_motion.destination(), remaining)
                    }
                }
                else {
                    (ball_motion.destination(), remaining)
                }
            };

            self.ball_pos = point;
            self.ball_vel.x = self.ball_vel.x
                .min( BALL_MAX_X_SPEED)
                .max(-BALL_MAX_X_SPEED);
            remaining -= travel;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::draw(ctx, &self.paddle_mesh, (na::Point2::new(self.paddle_x, PADDLE_Y),))?;
        graphics::draw(ctx, &self.ball_mesh, (self.ball_pos,))?;

        const BLOCK_COLORS: &[graphics::Color] = &[
            graphics::Color::new(1., 0., 1., 1.),
            graphics::Color::new(1., 0., 0., 1.),
            graphics::Color::new(1., 1., 0., 1.),
            graphics::Color::new(0., 1., 0., 1.),
            graphics::Color::new(0., 0., 1., 1.),
            graphics::Color::new(0., 1., 1., 1.),
        ];

        for block in self.blocks.iter() {
            let color = BLOCK_COLORS[block.hp as usize % BLOCK_COLORS.len()];

            let block_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                self.block_rect.into(),
                color
            )?;

            graphics::draw(ctx, &block_mesh, (block.pos,))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let window_mode = ggez::conf::WindowMode {
        width: GAME_WIDTH,
        height: GAME_HEIGHT,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
        resizable: false,
    };

    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("super_simple", "ggez")
        .window_mode(window_mode)
        .build()?;

    let screen_rect = graphics::Rect::new(-GAME_WIDTH * 0.5, GAME_HEIGHT, GAME_WIDTH, -GAME_HEIGHT);
    graphics::set_screen_coordinates(ctx, screen_rect)?;

    let state = &mut State::new(ctx)?;
    event::run(ctx, event_loop, state)
}

