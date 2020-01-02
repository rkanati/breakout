
mod block;
mod collider;
mod dilate;
mod game;
mod math;

use {
    crate::{
        block::Block,
        dilate::Dilate,
        math::*,
    },
    ggez::{
        self,
        event, graphics, input::keyboard, timer,
        Context, GameResult,
    },
    rand_core::RngCore,
};

const KEY_LEFT : keyboard::KeyCode = keyboard::KeyCode::A;
const KEY_RIGHT: keyboard::KeyCode = keyboard::KeyCode::D;
const KEY_SERVE: keyboard::KeyCode = keyboard::KeyCode::Space;

fn block_color(block: &Block) -> graphics::Color {
    use graphics::Color as C;
    const COLORS: &[graphics::Color] = &[
        C::new(0.0, 0.0, 0.1, 1.), // dark blue
        C::new(0.2, 0.0, 0.1, 1.), // dark purple
        C::new(0.3, 0.0, 0.2, 1.), // purple
        C::new(0.4, 0.0, 0.2, 1.), // red
        C::new(0.6, 0.0, 0.0, 1.), // red
        C::new(0.9, 0.0, 0.0, 1.), // red
        C::new(0.9, 0.2, 0.0, 1.), // orange
        C::new(0.9, 0.5, 0.0, 1.), // orange
        C::new(1.0, 0.8, 0.0, 1.), // yellow
        C::new(1.0, 1.0, 0.1, 1.), // yellow
        C::new(1.0, 1.0, 1.0, 1.), // white
    ];

    match block.hp() {
        Some(hp) => COLORS[(hp as usize - 1).min(COLORS.len() - 1)],
        None     => C::new(0.0, 0.0, 0.0, 1.)
    }
}

const FRAMERATE: u32 = 180;
const DT:        f32 = 1. / FRAMERATE as f32;

struct Sprite<'ctx> {
    pub mesh:  &'ctx graphics::Mesh,
    pub angle: f32,
}

impl<'ctx> Sprite<'ctx> {
    fn new(mesh: &'ctx graphics::Mesh, angle: f32) -> Sprite {
        Sprite { mesh, angle }
    }
}

struct App<'ctx> {
    state: game::State,

    font: graphics::Font,
    bonus: graphics::Mesh,
    heart: Sprite<'ctx>,
    bomb:  Sprite<'ctx>,
    multi: Sprite<'ctx>,
}

impl App {
    fn new(ctx: &mut Context, state: game::State) -> GameResult<App> {
        let font = graphics::Font::new(ctx, "/Signika-SemiBold.ttf")?;

        let heart_points = [
            P2::new( 0., -6.),
            P2::new(-6.,  0.),
            P2::new(-3.,  3.),
            P2::new( 0.,  0.),
            P2::new( 3.,  3.),
            P2::new( 6.,  0.),
        ];

        let heart = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::fill(),
            &heart_points,
            [1., 0., 0., 1.].into()
        )?;

        let game = App { state, font, heart };
        Ok(game)
    }

    fn pickup_sprite(&self, kind: game::PickupKind) -> Sprite {
        use game::PickupKind::*;
        match kind {
            Bonus(amount) => bonus_sprite(amount),
            ExtraBall => self.heart,
            Detonator => Sprite::new(15., 0., [1.0, 0.0, 0.0, 1.].into()),
            MultiBall => Sprite::new(15., 0., [0.0, 1.0, 0.0, 1.].into()),
        }
    }
}

fn bonus_sprite(amount: i32) -> Sprite {
}

impl event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let left  = keyboard::is_key_pressed(ctx, KEY_LEFT);
        let right = keyboard::is_key_pressed(ctx, KEY_RIGHT);

        let paddle_dir =
            if      left && !right { -1 }
            else if right && !left {  1 }
            else                   {  0 };

        let serve = keyboard::is_key_pressed(ctx, KEY_SERVE);

        let input = game::Input { paddle_dir, serve };

        while timer::check_update_time(ctx, FRAMERATE) {
            if !self.state.update(DT, input) {
                event::quit(ctx);
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let alpha = timer::remaining_update_time(ctx).as_secs_f32() * FRAMERATE as f32;
        let frame = self.state.frame(alpha);

        graphics::clear(ctx, [0.05, 0.10, 0.15, 1.].into());

        let paddle_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            frame.paddle_rect.into(),
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &paddle_mesh, (frame.paddle_pos,))?;

        let ball_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            frame.ball_rect.into(),
            graphics::Color::new(1.0, 0.5, 0.0, 1.),
        )?;
        graphics::draw(ctx, &ball_mesh, (frame.ball_pos,))?;

        for block in frame.blocks.iter() {
            let block_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                block.rect.contract(1.).into(),
                block_color(block)
            )?;

            graphics::draw(ctx, &block_mesh, (P2::new(0., 0.), ))?;
        }

        for pickup in frame.pickups {
            let sprite = pickup_sprite(pickup.kind);

            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new_square_centered(sprite.size).into(),
                sprite.color,
            )?;

            let params = graphics::DrawParam::new()
                .dest(pickup.position)
                .rotation(sprite.angle);

            graphics::draw(ctx, &mesh, params)?;
        }

        let status_line = format!(
            "Score: {:8} Combo: x{:1.1} {:+8}",
            frame.scoring.score,
            frame.scoring.combo_multiplier,
            frame.scoring.combo_score
        );

        let mut text = graphics::Text::new(status_line);
        text.set_font(self.font, graphics::Scale::uniform(20.));

        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(frame.rect.mins + V2::new(1., 21.))
                //.dest(P2::new(1., 21.))
                .scale(V2::new(1., -1.)),
        )?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }
}

pub fn main() -> GameResult {
    let seed = rand::rngs::OsRng.next_u64();
    let state = game::State::new(seed);
    let rect = state.rect();

    let window_mode = ggez::conf::WindowMode {
        width:  rect.width(),
        height: rect.height(),
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
        resizable: false,
    };

    let window_setup = ggez::conf::WindowSetup {
        title: "Breakout".to_owned(),
        samples: ggez::conf::NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: true,
    };

    let mut ctx_builder = ggez::ContextBuilder::new("breakout", "ggez");

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        ctx_builder = ctx_builder.add_resource_path(path);
    }

    let (ctx, event_loop) = &mut ctx_builder
        .window_mode(window_mode)
        .window_setup(window_setup)
        .build()?;

    let mut screen_rect = graphics::Rect::from(rect);
    screen_rect.y = screen_rect.h;
    screen_rect.h *= -1.;
    graphics::set_screen_coordinates(ctx, screen_rect)?;

    let app = &mut App::new(ctx, state)?;
    event::run(ctx, event_loop, app)
}

