use crate::GameName::{Invaders, Pong, Snake};
use crate::GameUpdateResult::{Nothing, Pop, Push};
use crate::{Game, GameUpdateResult, CLR_2, CLR_3};
use pixels_graphics_lib::buffer_graphics_lib::prelude::Positioning::LeftTop;
use pixels_graphics_lib::buffer_graphics_lib::prelude::TextPos::Px;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::graphics_shapes::triangle::FlatSide;
use pixels_graphics_lib::prelude::*;

const TITLE: &str = "Games";
const OPTIONS: [&str; 2] = ["Pong", "Snake"];//, "Invaders"];
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
        let title = Text::new(TITLE, TITLE_POS, (CLR_3, TextSize::Large, LeftTop));
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
                    (CLR_3, TextSize::Large, LeftTop),
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

    fn on_key_press(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowUp => {
                if self.cursor_idx == 0 {
                    self.cursor_idx = self.options.len() - 1;
                } else {
                    self.cursor_idx -= 1;
                }
            }
            KeyCode::ArrowDown => {
                if self.cursor_idx == self.options.len() - 1 {
                    self.cursor_idx = 0;
                } else {
                    self.cursor_idx += 1;
                }
            }
            KeyCode::Enter => match self.cursor_idx {
                0 => self.result = Push(Pong),
                1 => self.result = Push(Snake),
                2 => self.result = Push(Invaders),
                _ => {}
            },
            KeyCode::Escape => self.result = Pop,
            _ => {}
        }
    }

    fn update(&mut self, _: &Timing, _: &Vec<&KeyCode>) -> GameUpdateResult {
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
