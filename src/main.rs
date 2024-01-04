mod button_bar;
mod games;

use crate::games::invaders::Invaders;
use crate::games::menu::GameMenu;
use crate::games::pong::Pong;
use crate::games::snake::Snake;
use color_eyre::Result;
use log::LevelFilter;
use pixels_graphics_lib::buffer_graphics_lib::color::Color;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::buffer_graphics_lib::text::format::Positioning::LeftBottom;
use pixels_graphics_lib::buffer_graphics_lib::text::pos::TextPos;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Small;
use pixels_graphics_lib::buffer_graphics_lib::Graphics;
use pixels_graphics_lib::prelude::*;
use std::collections::HashSet;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 166;

const INPUT_DELAY: f64 = 0.2;

const CLR_3: Color = GB_3;
const CLR_2: Color = GB_2;
const CLR_1: Color = GB_1;
const CLR_0: Color = GB_0;

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
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        "Retro Games",
        system,
        Options {
            ups: 60,
            vsync: true,
            ..Options::default()
        },
    )?;
    Ok(())
}

struct GameHost {
    game_stack: Vec<Box<dyn Game>>,
    held_keys: HashSet<KeyCode>,
    controller: GameController,
    keyboard: bool,
}

impl GameHost {
    pub fn new() -> Self {
        Self {
            game_stack: vec![Box::new(GameMenu::new())],
            held_keys: HashSet::new(),
            controller: GameController::new_unchecked(),
            keyboard: false,
        }
    }
}

impl System for GameHost {
    fn keys_used(&self) -> &[KeyCode] {
        &[
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::Escape,
            KeyCode::Space,
        ]
    }

    fn window_prefs(&mut self) -> Option<WindowPreferences> {
        Some(WindowPreferences::new("app", "emmabritton", "retro_games", 3).unwrap())
    }

    fn update(&mut self, timing: &Timing) {
        self.controller.update();
        if self.controller.mask() != 0 {
            self.keyboard = false;
        }
        if let Some(game) = self.game_stack.last_mut() {
            match game.update(timing, &self.held_keys.iter().collect(), &self.controller) {
                GameUpdateResult::Nothing => {}
                GameUpdateResult::Push(new_game) => match new_game {
                    GameName::Pong => self.game_stack.push(Pong::new()),
                    GameName::Snake => self.game_stack.push(Snake::new()),
                    GameName::Invaders => self.game_stack.push(Invaders::new()),
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

    fn render(&mut self, graphics: &mut Graphics) {
        graphics.clear(CLR_0);
        if let Some(game) = self.game_stack.last() {
            let controller = if self.keyboard {
                None
            } else {
                self.controller.get_controller_type()
            };
            game.render(graphics, controller);
        }
        if cfg!(debug_assertions) {
            graphics.draw_text(
                "",
                TextPos::Px(0, SCREEN_HEIGHT as isize),
                (CLR_1, Small, LeftBottom),
            );
        }
    }

    fn on_key_down(&mut self, keys: Vec<KeyCode>) {
        self.keyboard = true;
        for key in keys {
            self.held_keys.insert(key);
        }
    }

    fn on_key_up(&mut self, keys: Vec<KeyCode>) {
        for key in &keys {
            self.held_keys.remove(key);
        }
        if let Some(game) = self.game_stack.last_mut() {
            for key in &keys {
                game.on_key_press(*key)
            }
        }
    }

    fn should_exit(&mut self) -> bool {
        self.game_stack.is_empty()
    }
}

trait Game {
    fn render(&self, graphics: &mut Graphics, controller: Option<Controller>);
    fn on_key_press(&mut self, key: KeyCode);
    #[allow(clippy::ptr_arg)] //breaks other code if changed
    fn update(
        &mut self,
        timing: &Timing,
        held_keys: &Vec<&KeyCode>,
        controller: &GameController,
    ) -> GameUpdateResult;
    fn resuming(&mut self);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameName {
    Pong,
    Snake,
    Invaders,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameUpdateResult {
    Nothing,
    Push(GameName),
    Pop,
}
