use std::ops::Neg;
use crate::{Game, GameUpdateResult, CLR_1, CLR_2, CLR_3, CLR_0, SCREEN_WIDTH, SCREEN_HEIGHT};
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::buffer_graphics_lib::shapes::CreateDrawable;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::{Center, LeftTop};
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Large;
use pixels_graphics_lib::prelude::*;
use crate::GameUpdateResult::{Nothing, Pop};
use crate::snake::Direction::*;
use crate::snake::State::*;

const TILE_SIZE: usize = 8;
const ARENA_WIDTH: usize = 18;
const ARENA_HEIGHT: usize = 14;
const ARENA_START: Coord = Coord::new(0, 16);
const FRUIT_DELAY: f64 = 5.0;

const SPEED_CHANGE_PER_TICK: f64 = 0.001;
const SPEED_CHANGE_PER_FRUIT: f64 = 0.005;
const DEFAULT_MOVE_SPEED: f64 = 0.4;
const MIN_MOVE_SPEED: f64 = 0.07;

const SCORE_PER_TICK: usize = 1;
const SCORE_PER_FRUIT: usize = 100;

const MAX_FRUIT_ON_SCREEN: usize = 3;
const DYING_ANIM_RATE: f64 = 0.1;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum State {
    Playing,
    Won,
    Dying,
    Dead
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    pub fn delta(&self) -> Coord {
        match self {
            Up => Coord::new(0,-1),
            Left => Coord::new(-1,0),
            Right => Coord::new(1,0),
            Down => Coord::new(0,1),
        }
    }
}

#[derive(Debug)]
pub struct Snake {
    fruit: Drawable<Circle>,
    segment: Drawable<Rect>,
    wall: Drawable<Rect>,
    body: Vec<Coord>,
    fruits: Vec<Coord>,
    next_fruit_spawn: f64,
    result: GameUpdateResult,
    move_speed: f64,
    next_move: f64,
    score: usize,
    state: State,
    direction: Direction,
    next_dying_anim: f64
}

impl Snake {
    pub fn new() -> Box<Self> {
        let fruit = Drawable::from_obj(
            Circle::new((TILE_SIZE / 2, TILE_SIZE / 2), TILE_SIZE/ 2-1),
            fill(CLR_3),
        );
        let segment = Drawable::from_obj(
            Rect::new((0, 0), (TILE_SIZE - 2, TILE_SIZE - 2)),
            fill(CLR_3),
        );
        let wall = Drawable::from_obj(
            Rect::new((0, 0), (TILE_SIZE - 2, TILE_SIZE - 2)),
            fill(CLR_1),
        );
        Box::new(Self {
            fruit,
            segment,
            wall,
            body: vec![Coord::new(9, 7), Coord::new(8, 7), Coord::new(7,7)],
            fruits: vec![],
            next_fruit_spawn: FRUIT_DELAY / 3.0,
            next_move: 0.0,
            move_speed: DEFAULT_MOVE_SPEED,
            score: 0,
            result: Nothing,
            state: Playing,
            next_dying_anim: DYING_ANIM_RATE,
            direction: Right
        })
    }
}

impl Snake {
    fn find_empty_slot(&self) -> Option<Coord> {
        for _ in 0..200 {
            let x = fastrand::usize(1..ARENA_WIDTH);
            let y = fastrand::usize(1..ARENA_HEIGHT);
            let xy = Coord::from((x,y));
            if !self.fruits.contains(&xy) && !self.body.contains(&xy) {
                return Some(xy);
            }
        }
        None
    }
}

impl Game for Snake {
    fn render(&self, graphics: &mut Graphics) {
        graphics.update_translate(ARENA_START + (1,0));

        let wall_horz_size = ARENA_WIDTH + 1;
        let wall_vert_size = ARENA_HEIGHT + 1;

        for x in 0..=wall_horz_size {
            graphics.draw(&self.wall.with_move(((x * TILE_SIZE) as isize, 0)));
            graphics.draw(&self.wall.with_move(((x * TILE_SIZE) as isize , (wall_vert_size * TILE_SIZE) as isize)));
        }
        for y in 0..=wall_vert_size {
            graphics.draw(&self.wall.with_move((0,(y * TILE_SIZE) as isize )));
            graphics.draw(&self.wall.with_move(((wall_horz_size * TILE_SIZE) as isize, (y*TILE_SIZE) as isize)));
        }

        for segment in &self.body {
            let xy = *segment * (TILE_SIZE, TILE_SIZE);
            graphics.draw(&self.segment.with_move(xy));
        }

        for fruit in &self.fruits {
            let xy = *fruit * (TILE_SIZE, TILE_SIZE);
            graphics.draw(&self.fruit.with_move(xy + (TILE_SIZE/2,TILE_SIZE/2)));
        }

        graphics.update_translate((ARENA_START + (1,0)).neg());

        graphics.draw_text(&format!("Score: {: >8}", self.score), TextPos::Px(3,3), (if self.state != Playing {CLR_3} else {CLR_2}, Large, LeftTop));

        match self.state {
            Playing => {}
            Won => {
                let x1 = 35;
                let y1 = 67;
                let x2 = 120;
                let y2 = 90;
                graphics.draw_rect(Rect::new((x1,y1),(x2,y2)), fill(CLR_0));
                graphics.draw_rect(Rect::new((x1,y1),(x2,y2)), stroke(CLR_3));
                graphics.draw_rect(Rect::new((x1+1,y1+1),(x2-1,y2-1)), stroke(CLR_2));
                graphics.draw_rect(Rect::new((x1+2,y1+2),(x2-2,y2-2)), stroke(CLR_1));
                graphics.draw_text("You win!", TextPos::Px((SCREEN_WIDTH as isize - ARENA_START.x) /2, (SCREEN_HEIGHT as isize - ARENA_START.y) / 2 + ARENA_START.y ), (CLR_3, Large, Center));
            }
            Dying => {}
            Dead => {
                let x1 = 15;
                let y1 = 67;
                let x2 = 140;
                let y2 = 90;
                graphics.draw_rect(Rect::new((x1,y1),(x2,y2)), fill(CLR_0));
                graphics.draw_rect(Rect::new((x1,y1),(x2,y2)), stroke(CLR_3));
                graphics.draw_rect(Rect::new((x1+1,y1+1),(x2-1,y2-1)), stroke(CLR_2));
                graphics.draw_rect(Rect::new((x1+2,y1+2),(x2-2,y2-2)), stroke(CLR_1));
                graphics.draw_text("You're dead!", TextPos::Px((SCREEN_WIDTH as isize - ARENA_START.x) /2, (SCREEN_HEIGHT as isize - ARENA_START.y) / 2 + ARENA_START.y ), (CLR_3, Large, Center));
            }
        }
    }

    fn on_key_press(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Up => {
                let next =  self.body[0] + Up.delta();
                if self.body[1] != next {
                    self.direction = Up;
                }
            }
            VirtualKeyCode::Left => {
                let next =  self.body[0] + Left.delta();
                if self.body[1] != next {
                    self.direction = Left;
                }
            }
            VirtualKeyCode::Right => {
                let next =  self.body[0] + Right.delta();
                if self.body[1] != next {
                    self.direction = Right;
                }
            }
            VirtualKeyCode::Down => {
                let next =  self.body[0] + Down.delta();
                if self.body[1] != next {
                    self.direction = Down;
                }
            }
            VirtualKeyCode::Escape => self.result = Pop,
            _ => {}
        }
    }

    fn update(&mut self, timing: &Timing, _: &Vec<&VirtualKeyCode>) -> GameUpdateResult {
        match self.state {
            Playing => {
                if self.body.len() == (ARENA_HEIGHT * ARENA_WIDTH) {
                    self.score += 1000;
                    self.state = Won;
                    return self.result;
                }

                if self.fruits.len() < MAX_FRUIT_ON_SCREEN {
                    if self.next_fruit_spawn < 0.0 {
                        if let Some(empty) = self.find_empty_slot() {
                            self.fruits.push(empty);
                            self.next_fruit_spawn = FRUIT_DELAY;
                        }
                    }
                    self.next_fruit_spawn -= timing.fixed_time_step;
                }

                if self.next_move < 0.0 {
                    let next_tile = self.body[0] + self.direction.delta();
                    if next_tile.x == 0 || next_tile.y == 0 || next_tile.x == ARENA_WIDTH as isize +1 || next_tile.y == ARENA_HEIGHT as isize+1 ||
                        self.body.contains(&next_tile) {
                        self.state = Dying;
                        return self.result;
                    }

                    if let Some(i) = self.fruits.iter().position(|fruit| fruit == &next_tile) {
                        self.fruits.remove(i);
                        self.body.insert(0, next_tile);
                        self.next_move = self.move_speed;
                        self.score += SCORE_PER_FRUIT;
                        self.move_speed -= SPEED_CHANGE_PER_FRUIT;
                        if self.fruits.is_empty() {
                            self.next_fruit_spawn = 0.2;
                        }
                    } else {
                        self.body.remove(self.body.len() - 1);
                        self.body.insert(0, next_tile);
                        self.next_move = self.move_speed;
                        self.score += SCORE_PER_TICK;
                        self.move_speed -= SPEED_CHANGE_PER_TICK;
                    }
                    self.move_speed = self.move_speed.max(MIN_MOVE_SPEED);
                }
                self.next_move -= timing.fixed_time_step;
            }
            Won => {}
            Dying => {
                self.fruits.clear();
                if self.next_dying_anim < 0.0 {
                    if self.body.is_empty() {
                       self.state = Dead;
                    } else {
                        self.body.remove(self.body.len() - 1);
                        self.next_dying_anim = DYING_ANIM_RATE;
                    }
                }
                self.next_dying_anim -= timing.fixed_time_step;
            }
            Dead => {}
        }

        self.result
    }

    fn resuming(&mut self) {

    }
}
