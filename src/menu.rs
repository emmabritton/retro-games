use crate::GameName::{Pong, Snake};
use crate::GameUpdateResult::{Nothing, Pop, Push};
use crate::{Game, GameUpdateResult, CLR_2, CLR_3};
use pixels_graphics_lib::buffer_graphics_lib::drawable::{fill, Drawable};
use pixels_graphics_lib::buffer_graphics_lib::shapes::collection::ShapeCollection;
use pixels_graphics_lib::buffer_graphics_lib::shapes::CreateDrawable;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::LeftTop;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos::Px;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Large;
use pixels_graphics_lib::buffer_graphics_lib::text::{large, Text};
use pixels_graphics_lib::buffer_graphics_lib::Graphics;
use pixels_graphics_lib::graphics_shapes::triangle::FlatSide;
use pixels_graphics_lib::prelude::*;
use winit::event::VirtualKeyCode;

const TITLE: &str = "Games";
const OPTIONS: [&str; 2] = ["Pong", "Snake"];
const TITLE_POS: TextPos = Px(8, 8);
const CURSOR_X: isize = 8;
const MENU_X: isize = 20;
const MENU_START_Y: isize = 30;
const MENU_STEP: usize = large::CHAR_HEIGHT + 4;

pub struct GameMenu {
    title: Text,
    cursor_idx: usize,
    cursor: Drawable<Triangle>,
    options: Vec<Text>,
    frame: ShapeCollection,
    result: GameUpdateResult,
}

impl GameMenu {
    pub fn new() -> Self {
        let title = Text::new(TITLE, TITLE_POS, (CLR_3, Large, LeftTop));
        let cursor = Drawable::from_obj(
            Triangle::equilateral((CURSOR_X + 3, MENU_START_Y + 3), 6, FlatSide::Left),
            fill(CLR_3),
        );
        let frame = ShapeCollection::new();
        let options = OPTIONS
            .iter()
            .enumerate()
            .map(|(i, text)| {
                Text::new(
                    text,
                    Px(MENU_X, MENU_START_Y + (i * MENU_STEP) as isize),
                    (CLR_3, Large, LeftTop),
                )
            })
            .collect();

        Self {
            title,
            cursor_idx: 0,
            cursor,
            frame,
            options,
            result: Nothing,
        }
    }
}

impl Game for GameMenu {
    fn render(&self, graphics: &mut Graphics) {
        graphics.draw(&self.frame);
        graphics.draw(&self.title);
        graphics.draw(&self.cursor);
        for (i, option) in self.options.iter().enumerate() {
            let color = if self.cursor_idx == i { CLR_3 } else { CLR_2 };
            graphics.draw(&option.with_color(color));
        }
    }

    fn on_key_press(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Up => {
                if self.cursor_idx == 0 {
                    self.cursor_idx = self.options.len() - 1;
                } else {
                    self.cursor_idx -= 1;
                }
            }
            VirtualKeyCode::Down => {
                if self.cursor_idx == self.options.len() - 1 {
                    self.cursor_idx = 0;
                } else {
                    self.cursor_idx += 1;
                }
            }
            VirtualKeyCode::Return => match self.cursor_idx {
                0 => self.result = Push(Pong),
                1 => self.result = Push(Snake),
                _ => {}
            },
            VirtualKeyCode::Escape => self.result = Pop,
            _ => {}
        }
    }

    fn update(&mut self, _: &Timing, _: &Vec<&VirtualKeyCode>) -> GameUpdateResult {
        self.cursor = self.cursor.with_move((
            CURSOR_X,
            MENU_START_Y + 1 + (self.cursor_idx * MENU_STEP) as isize,
        ));

        self.result
    }

    fn resuming(&mut self) {
        self.result = Nothing;
    }
}
