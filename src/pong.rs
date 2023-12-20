use crate::pong::Direction::*;
use crate::GameUpdateResult::{Nothing, Pop};
use crate::{Game, GameUpdateResult, CLR_2, CLR_3, SCREEN_HEIGHT, SCREEN_WIDTH, INPUT_DELAY};
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::buffer_graphics_lib::shapes::CreateDrawable;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::CenterTop;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::Text;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Large;
use pixels_graphics_lib::prelude::*;

const PADDLE_X_H: usize = 0;
const PADDLE_X_C: usize = SCREEN_WIDTH - 6;
const SCORE_H: Coord = Coord::new(40, 6);
const SCORE_C: Coord = Coord::new(120, 6);
const BALL_MOVE_RATE: f64 = 0.01;
const PADDLE_MOVE_DISTANCE: isize = 2;

#[derive(Debug)]
struct Player {
    paddle: Drawable<Rect>,
    next_move: Timer,
    score: usize,
}

impl Player {
    pub fn new(x: usize) -> Self {
        Self {
            paddle: Drawable::from_obj(Rect::new((x, 0), (x + 6, 30)), fill(CLR_3)),
            next_move: Timer::new_once(0.0001),
            score: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Debug)]
struct Ball {
    shape: Drawable<Circle>,
    direction: isize,
    next_move: f64,
    last_bounce_side: Direction,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            last_bounce_side: Left,
            shape: Drawable::from_obj(Circle::new((40, 40), 5), fill(CLR_3)),
            direction: 135,
            next_move: 0.0,
        }
    }
}

pub struct Pong {
    human: Player,
    cpu: Player,
    ball: Ball,
    separator: Drawable<Rect>,
    serving: bool,
    result: GameUpdateResult,
    audio_engine: AudioEngine,
    paddle: SoundEffect,
    miss: SoundEffect,
    wall: SoundEffect,
    input_timer: Timer,
    controller: GameController,
}

impl Pong {
    pub fn new() -> Box<Self> {
        let audio_engine = AudioEngine::new().unwrap();
        let wall = audio_engine
            .load_from_bytes(include_bytes!("../assets/wall.wav"), 0.2)
            .unwrap();
        let paddle = audio_engine
            .load_from_bytes(include_bytes!("../assets/paddle.wav"), 0.2)
            .unwrap();
        let miss = audio_engine
            .load_from_bytes(include_bytes!("../assets/ball.wav"), 0.4)
            .unwrap();

        let separator = Drawable::from_obj(
            Rect::new(
                (SCREEN_WIDTH / 2 - 1, 0),
                (SCREEN_WIDTH / 2 + 1, SCREEN_HEIGHT),
            ),
            fill(CLR_2),
        );
        Box::new(Self {
            result: Nothing,
            paddle,
            miss,
            serving: true,
            human: Player::new(PADDLE_X_H),
            cpu: Player::new(PADDLE_X_C),
            ball: Ball::new(),
            separator,
            wall,
            audio_engine,
            controller: GameController::new_unchecked(),
            input_timer: Timer::new(INPUT_DELAY),
        })
    }
}

impl Pong {
    fn reset_play(&mut self) {
        self.serving = true;
        self.ball.shape = self.ball.shape.with_move((40, fastrand::isize(40..100)));
        self.human.paddle = self.human.paddle.with_move((PADDLE_X_H, SCREEN_HEIGHT / 2));
        self.cpu.paddle = self.cpu.paddle.with_move((PADDLE_X_C, SCREEN_HEIGHT / 2));
        self.ball.last_bounce_side = Left;
        self.ball.direction = [45, 135][fastrand::usize(0..=1)]
    }
}

impl Game for Pong {
    fn render(&self, graphics: &mut Graphics) {
        self.separator.render(graphics);

        graphics.draw(&Text::new(
            &format!("{}", self.human.score),
            TextPos::px(SCORE_H),
            (CLR_2, Large, CenterTop),
        ));
        graphics.draw(&Text::new(
            &format!("{}", self.cpu.score),
            TextPos::px(SCORE_C),
            (CLR_2, Large, CenterTop),
        ));

        self.human.paddle.render(graphics);
        self.cpu.paddle.render(graphics);
        self.ball.shape.render(graphics);
    }

    fn on_key_press(&mut self, _: KeyCode) {

    }

    #[allow(clippy::collapsible_if)] //for readability
    fn update(&mut self, timing: &Timing, held_keys: &Vec<&KeyCode>) -> GameUpdateResult {
        self.controller.update();
        self.wall.update(timing);
        self.paddle.update(timing);
        self.miss.update(timing);

        if self.serving && (held_keys.contains(&& KeyCode::Space) || self.controller.action.south){
            self.serving = false
        }

        if held_keys.contains(&& KeyCode::Escape ) || self.controller.action.east{
            self.result = Pop;
        }

        if self.human.next_move.update(timing) {
            if held_keys.contains(&&KeyCode::ArrowUp) || self.controller.direction.up {
                if self.human.paddle.obj().top() > 0 {
                    self.human.paddle = self
                        .human
                        .paddle
                        .with_translation((0, -PADDLE_MOVE_DISTANCE));
                    self.human.next_move.reset();
                }
            } else if held_keys.contains(&&KeyCode::ArrowDown) || self.controller.direction.down {
                if self.human.paddle.obj().bottom() < SCREEN_HEIGHT as isize {
                    self.human.paddle = self
                        .human
                        .paddle
                        .with_translation((0, PADDLE_MOVE_DISTANCE));
                    self.human.next_move.reset();
                }
            }
        }

        let ball_center = self.ball.shape.obj().center();
        if !self.serving {
            if self.ball.next_move <= 0.0 {
                let next_coord = Coord::from_angle(ball_center, 1, self.ball.direction);
                self.ball.shape = self.ball.shape.with_move(next_coord);
                self.ball.next_move = BALL_MOVE_RATE;

                if ball_center.y == 0 {
                    if self.ball.last_bounce_side != Top {
                        self.ball.direction = if self.ball.direction == 45 { 135 } else { 225 };
                        self.ball.last_bounce_side = Top;
                        self.wall.play();
                    }
                } else if ball_center.y == SCREEN_HEIGHT as isize {
                    if self.ball.last_bounce_side != Bottom {
                        self.ball.direction = if self.ball.direction == 135 { 45 } else { 315 };
                        self.ball.last_bounce_side = Bottom;
                        self.wall.play();
                    }
                } else if self.human.paddle.obj().contains(ball_center) {
                    if self.ball.last_bounce_side != Left {
                        self.ball.direction = if self.ball.direction == 315 { 45 } else { 135 };
                        self.ball.last_bounce_side = Left;
                        self.paddle.play();
                    }
                } else if self.cpu.paddle.obj().contains(ball_center) {
                    if self.ball.last_bounce_side != Right {
                        self.ball.direction = if self.ball.direction == 135 { 225 } else { 315 };
                        self.ball.last_bounce_side = Right;
                        self.paddle.play();
                    }
                } else if ball_center.x == 0 {
                    self.cpu.score += 1;
                    self.reset_play();
                    self.miss.play();
                } else if ball_center.x == SCREEN_WIDTH as isize {
                    self.human.score += 1;
                    self.reset_play();
                    self.miss.play();
                }
            }
            self.ball.next_move -= timing.fixed_time_step;

            if self.cpu.next_move.update(timing) {
                if fastrand::bool() {
                    let cpu_center = self.cpu.paddle.obj().center();
                    if cpu_center.y < ball_center.y
                        && self.cpu.paddle.obj().bottom() < SCREEN_HEIGHT as isize
                    {
                        self.cpu.paddle =
                            self.cpu.paddle.with_translation((0, PADDLE_MOVE_DISTANCE));
                        self.cpu.next_move.reset();
                    } else if cpu_center.y > ball_center.y && self.cpu.paddle.obj().top() > 0 {
                        self.cpu.paddle =
                            self.cpu.paddle.with_translation((0, -PADDLE_MOVE_DISTANCE));
                        self.cpu.next_move.reset();
                    }
                }
            }
        }

        self.result
    }

    fn resuming(&mut self) {}
}
