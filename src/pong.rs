use crate::pong::Direction::*;
use crate::GameUpdateResult::{Nothing, Pop};
use crate::{Game, GameUpdateResult, CLR_2, CLR_3, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::buffer_graphics_lib::shapes::CreateDrawable;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::CenterTop;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::Text;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Large;
use pixels_graphics_lib::Timing;
use winit::event::VirtualKeyCode;

const PADDLE_X_H: usize = 0;
const PADDLE_X_C: usize = SCREEN_WIDTH - 6;
const SCORE_H: Coord = Coord::new(40, 6);
const SCORE_C: Coord = Coord::new(120, 6);
const PADDLE_MOVE_RATE: f64 = 0.01;
const BALL_MOVE_RATE: f64 = 0.001;

#[derive(Debug)]
struct Player {
    paddle: Drawable<Rect>,
    next_move: f64,
    score: usize,
}

impl Player {
    pub fn new(x: usize) -> Self {
        Self {
            paddle: Drawable::from_obj(Rect::new((x, 0), (x + 6, 30)), fill(CLR_3)),
            next_move: 0.0,
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

#[derive(Debug)]
pub struct Pong {
    human: Player,
    cpu: Player,
    ball: Ball,
    separator: Drawable<Rect>,
    serving: bool,
    result: GameUpdateResult,
}

impl Pong {
    pub fn new() -> Box<Self> {
        let separator = Drawable::from_obj(
            Rect::new(
                (SCREEN_WIDTH / 2 - 1, 0),
                (SCREEN_WIDTH / 2 + 1, SCREEN_HEIGHT),
            ),
            fill(CLR_2),
        );
        Box::new(Self {
            result: Nothing,
            serving: true,
            human: Player::new(PADDLE_X_H),
            cpu: Player::new(PADDLE_X_C),
            ball: Ball::new(),
            separator,
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

    fn on_key_press(&mut self, key: VirtualKeyCode) {
        if self.serving && key == VirtualKeyCode::Space {
            self.serving = false
        }

        if key == VirtualKeyCode::Escape {
            self.result = Pop;
        }
    }

    fn update(&mut self, timing: &Timing, held_keys: &Vec<&VirtualKeyCode>) -> GameUpdateResult {
        let time_since_start = timing.now.duration_since(timing.started_at).as_secs_f64();
        if self.human.next_move <= time_since_start {
            if held_keys.contains(&&VirtualKeyCode::Up) {
                if self.human.paddle.obj().top() > 0 {
                    self.human.paddle = self.human.paddle.with_translation((0, -1));
                    self.human.next_move = time_since_start + PADDLE_MOVE_RATE;
                }
            } else if held_keys.contains(&&VirtualKeyCode::Down) {
                if self.human.paddle.obj().bottom() < SCREEN_HEIGHT as isize {
                    self.human.paddle = self.human.paddle.with_translation((0, 1));
                    self.human.next_move = time_since_start + PADDLE_MOVE_RATE;
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
                    }
                } else if ball_center.y == SCREEN_HEIGHT as isize {
                    if self.ball.last_bounce_side != Bottom {
                        self.ball.direction = if self.ball.direction == 135 { 45 } else { 315 };
                        self.ball.last_bounce_side = Bottom;
                    }
                } else if self.human.paddle.obj().contains(ball_center) {
                    if self.ball.last_bounce_side != Left {
                        self.ball.direction = if self.ball.direction == 315 { 45 } else { 135 };
                        self.ball.last_bounce_side = Left;
                    }
                } else if self.cpu.paddle.obj().contains(ball_center) {
                    if self.ball.last_bounce_side != Right {
                        self.ball.direction = if self.ball.direction == 135 { 225 } else { 315 };
                        self.ball.last_bounce_side = Right;
                    }
                } else if ball_center.x == 0 {
                    self.cpu.score += 1;
                    self.reset_play();
                } else if ball_center.x == SCREEN_WIDTH as isize {
                    self.human.score += 1;
                    self.reset_play();
                }
            }
            self.ball.next_move -= timing.fixed_time_step;

            if self.cpu.next_move <= time_since_start {
                if fastrand::bool() {
                    let cpu_center = self.cpu.paddle.obj().center();
                    if cpu_center.y < ball_center.y
                        && self.cpu.paddle.obj().bottom() < SCREEN_HEIGHT as isize
                    {
                        self.cpu.paddle = self.cpu.paddle.with_translation((0, 1));
                        self.cpu.next_move = time_since_start + PADDLE_MOVE_RATE;
                    } else if cpu_center.y > ball_center.y && self.cpu.paddle.obj().top() > 0 {
                        self.cpu.paddle = self.cpu.paddle.with_translation((0, -1));
                        self.cpu.next_move = time_since_start + PADDLE_MOVE_RATE;
                    }
                }
            }
        }

        self.result
    }

    fn resuming(&mut self) {}
}
