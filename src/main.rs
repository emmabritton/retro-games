mod menu;
mod pong;
mod snake;

use crate::menu::GameMenu;
use crate::pong::Pong;
use crate::snake::Snake;
use color_eyre::Result;
use pixels_graphics_lib::buffer_graphics_lib::color::Color;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::LeftBottom;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Small;
use pixels_graphics_lib::prefs::WindowPreferences;
use pixels_graphics_lib::prelude::*;
use std::collections::HashSet;
use log::LevelFilter;
use winit::event::VirtualKeyCode;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const TILES_SIZE: usize = 8;
const TILES_HORZ: usize = 20;
const TILES_VERT: usize = 18;

const CLR_3: Color = Color {
    r: 15,
    g: 56,
    b: 15,
    a: 255,
};

const CLR_2: Color = Color {
    r: 48,
    g: 98,
    b: 48,
    a: 255,
};

const CLR_1: Color = Color {
    r: 139,
    g: 172,
    b: 15,
    a: 255,
};

const CLR_0: Color = Color {
    r: 155,
    g: 188,
    b: 15,
    a: 255,
};

const COLORS: [Color; 4] = [CLR_0, CLR_1, CLR_2, CLR_3];

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::Builder::new()
        .filter_level(LevelFilter::Warn)
        .filter_module("games", LevelFilter::Trace)
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(false)
        .init();

    let system = Box::new(GameHost::new());
    run(
        160,
        144,
        WindowScaling::Auto,
        "Games",
        system,
        ExecutionSpeed::new(60),
    )?;
    Ok(())
}

struct GameHost {
    game_stack: Vec<Box<dyn Game>>,
    held_keys: HashSet<VirtualKeyCode>,
}

impl GameHost {
    pub fn new() -> Self {
        Self {
            game_stack: vec![Box::new(GameMenu::new())],
            held_keys: HashSet::new(),
        }
    }
}

impl System for GameHost {
    fn action_keys(&self) -> Vec<VirtualKeyCode> {
        vec![
            VirtualKeyCode::Up,
            VirtualKeyCode::Down,
            VirtualKeyCode::Left,
            VirtualKeyCode::Right,
            VirtualKeyCode::Escape,
            VirtualKeyCode::Space,
            VirtualKeyCode::Return,
            VirtualKeyCode::RControl,
            VirtualKeyCode::LControl,
        ]
    }

    fn window_prefs(&self) -> Option<WindowPreferences> {
        Some(WindowPreferences::new("app", "emmabritton", "retro_games").unwrap())
    }

    fn update(&mut self, timing: &Timing) {
        if let Some(game) = self.game_stack.last_mut() {
            match game.update(timing, &self.held_keys.iter().collect()) {
                GameUpdateResult::Nothing => {}
                GameUpdateResult::Push(new_game) => match new_game {
                    GameName::Pong => self.game_stack.push(Pong::new()),
                    GameName::Snake => self.game_stack.push(Snake::new()),
                },
                GameUpdateResult::Pop => {
                    self.game_stack.remove(self.game_stack.len() - 1);
                    if let Some(game) = self.game_stack.last_mut() {
                        game.resuming();
                    }
                }
            }
        }
    }

    fn render(&self, graphics: &mut Graphics) {
        graphics.clear(CLR_0);
        if let Some(game) = self.game_stack.last() {
            game.render(graphics);
        }
        let txt = self
            .held_keys
            .iter()
            .map(|k| format!("{k:?}"))
            .collect::<Vec<String>>()
            .join(", ");
        graphics.draw_text(
            &txt,
            TextPos::Px(0, SCREEN_HEIGHT as isize),
            (CLR_1, Small, LeftBottom),
        );
    }

    fn on_key_pressed(&mut self, keys: Vec<VirtualKeyCode>) {
        if let Some(game) = self.game_stack.last_mut() {
            for key in keys {
                game.on_key_press(key)
            }
        }
    }

    fn on_key_down(&mut self, keys: Vec<VirtualKeyCode>) {
        for key in keys {
            self.held_keys.insert(key);
        }
    }

    fn on_key_up(&mut self, keys: Vec<VirtualKeyCode>) {
        for key in keys {
            self.held_keys.remove(&key);
        }
    }

    fn should_exit(&self) -> bool {
        self.game_stack.is_empty()
    }
}

trait Game {
    fn render(&self, graphics: &mut Graphics);
    fn on_key_press(&mut self, key: VirtualKeyCode);
    fn update(&mut self, timing: &Timing, held_keys: &Vec<&VirtualKeyCode>) -> GameUpdateResult;
    fn resuming(&mut self);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameName {
    Pong,
    Snake,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameUpdateResult {
    Nothing,
    Push(GameName),
    Pop,
}
