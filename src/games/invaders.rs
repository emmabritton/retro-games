use crate::GameUpdateResult::{Nothing, Pop};
use crate::{Game, GameUpdateResult, CLR_1, CLR_2, CLR_3, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels_graphics_lib::prelude::Positioning::{RightBottom, RightTop};
use pixels_graphics_lib::prelude::TextPos::Px;
use pixels_graphics_lib::prelude::WrappingStrategy::SpaceBeforeCol;
use pixels_graphics_lib::prelude::*;

const CLR_SHIP: Color = CLR_3;
const CLR_ATTACK: Color = CLR_3;
const CLR_TEXT: Color = CLR_2;
const CLR_BASE: Color = CLR_2;
const PLAYER_Y: usize = 133;
const SCORE_XY: TextPos = Px(160, 1);
const HEART_XY: Coord = Coord::new(1, 1);
const HEART_SPACE: isize = 11;
const ALIEN_START: Coord = Coord::new(3, 18);
const BASE_START: Coord = Coord::new(10, 100);
const BASE_SPACE: isize = 40;
const PLAYER_SPEED: f64 = 0.01;
const PLAYER_ATTACK_SPEED: f64 = 0.006;
const ALIEN_ATTACK_SPEED: f64 = 0.1;
const PLAYER_ATTACK_RATE: f64 = 1.0;
const ALIEN_ATTACK_RATE: f64 = 5.0;
const MAX_PLAYER_ATTACKS: usize = 2;
const MAX_ALIENS_ATTACKS: usize = 4;
const UFO_SPEED: f64 = 0.07;
const UFO_RATE: f64 = 20.0;
const SCORE_UFO: usize = 1000;
const SCORE_INVADER: usize = 50;
const ALIEN_SPEED_START: f64 = 1.0;
const INVADER_SPEED_MIN: f64 = 0.1;
const SPEED_DELTA_PER_INVADER: f64 = 0.04;
const SPEED_DELTA_PER_LEVEL: f64 = 0.1;
const ALIEN_SIZE: (usize, usize) = (11, 8);
const ALIEN_SPACING: (usize, usize) = (4, 4);
const ALIEN_ROW_SIZE: usize = 9;
const UFO_START: Coord = Coord::new(-50, 50);

struct Player {
    ship: ShapeCollection,
    attacks: Vec<PlayerAttack>,
    next_move: f64,
    next_attack: Timer,
    attack: ShapeCollection,
    attack_sound: SoundEffect,
    death_sound: SoundEffect,
}

struct PlayerAttack {
    xy: Coord,
    next_move: f64,
}

struct EnemyAttack {
    xy: Coord,
    next_move: f64,
}

impl EnemyAttack {
    pub fn bounds(&self) -> Rect {
        Rect::new_with_size(self.xy, 5, 8)
    }
}

struct Base {
    offset: Coord,
    blocks: Vec<Vec<bool>>,
}

struct Aliens {
    ships: Vec<AnimatedIndexedImage>,
    alive: Vec<Vec<bool>>,
    offset: Coord,
    last_move: f64,
    dir: isize,
    move_rate: f64,
    death_sound: SoundEffect,
    move_sounds: [SoundEffect; 2],
    next_move_sound: usize,
    attacks: Vec<EnemyAttack>,
    next_attack: f64,
    attack: AnimatedIndexedImage,
}

impl Aliens {
    #[inline]
    fn coord_for_ship(&self, y: usize, x: usize) -> Coord {
        self.offset
            + (
                x * (ALIEN_SPACING.0 + ALIEN_SIZE.0),
                y * (ALIEN_SPACING.1 + ALIEN_SIZE.1),
            )
    }

    #[inline]
    fn rect_for_ship(&self, y: usize, x: usize) -> Rect {
        Rect::new_with_size(
            self.offset
                + (
                    x * (ALIEN_SPACING.0 + ALIEN_SIZE.0),
                    y * (ALIEN_SPACING.1 + ALIEN_SIZE.1),
                ),
            ALIEN_SIZE.0,
            ALIEN_SIZE.1,
        )
    }

    fn alive_count(&self) -> usize {
        self.alive
            .iter()
            .map(|r| r.iter().fold(0, |a, v| if *v { a + 1 } else { a }))
            .sum()
    }

    fn update(&mut self, timing: &Timing) {
        if self.last_move < 0.0 {
            let min = self.left_most_x();
            let max = self.right_most_x();
            if min.is_none() || max.is_none() {
                return;
            }
            let min = min.unwrap();
            let max = max.unwrap();
            self.offset = self.offset + (self.dir, 0);
            if min <= 0 {
                self.dir = 1;
                self.offset = self.offset + (0, 1);
            }
            if max >= SCREEN_WIDTH as isize {
                self.dir = -1;
                self.offset = self.offset + (0, 1);
            }
            self.last_move = self.move_rate;
            if self.move_sounds[self.next_move_sound].can_play() {
                self.move_sounds[self.next_move_sound].play();
                self.next_move_sound = if self.next_move_sound == 1 { 0 } else { 1 };
            }
        }
        self.last_move -= timing.fixed_time_step;
    }

    fn random_attack_position(&self) -> Option<Coord> {
        let mut positions: Vec<Coord> = self
            .alive
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, alive)| {
                    if *alive {
                        Some(self.coord_for_ship(y, x) + (ALIEN_SIZE.0 / 2, ALIEN_SIZE.1))
                    } else {
                        None
                    }
                })
            })
            .collect();
        if positions.is_empty() {
            return None;
        }
        let i = fastrand::usize(0..positions.len());
        Some(positions.remove(i))
    }

    fn left_most_x(&self) -> Option<isize> {
        self.alive
            .iter()
            .enumerate()
            .filter_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(|(x, alive)| {
                        if *alive {
                            Some(self.coord_for_ship(y, x).x)
                        } else {
                            None
                        }
                    })
                    .min()
            })
            .min()
            .map(|x| x - 2)
    }

    fn right_most_x(&self) -> Option<isize> {
        self.alive
            .iter()
            .enumerate()
            .filter_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(|(x, alive)| {
                        if *alive {
                            Some(self.coord_for_ship(y, x).x)
                        } else {
                            None
                        }
                    })
                    .max()
            })
            .max()
            .map(|x| x + ALIEN_SIZE.0 as isize + 2)
    }

    fn bottom_most_y(&self) -> Option<isize> {
        self.alive
            .iter()
            .enumerate()
            .filter_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(|(x, alive)| {
                        if *alive {
                            Some(self.coord_for_ship(y, x).y)
                        } else {
                            None
                        }
                    })
                    .max()
            })
            .max()
            .map(|y| y + ALIEN_SIZE.1 as isize)
    }

    fn detect_player_attack_hit(&mut self, xy: Coord) -> bool {
        for (y, row) in self.alive.iter().enumerate() {
            for (x, alive) in row.iter().enumerate() {
                if *alive && self.rect_for_ship(y, x).contains(xy) {
                    self.alive[y][x] = false;
                    self.move_rate = self.move_rate.min(self.move_rate - SPEED_DELTA_PER_INVADER);
                    return true;
                }
            }
        }
        false
    }
}

struct Ufo {
    sprite: AnimatedIndexedImage,
    xy: Coord,
    next_move: f64,
    next_appearance: f64,
    is_visible: bool,
    active_sound: SoundEffect,
    death_sound: SoundEffect,
}

pub struct Invaders {
    player: Player,
    aliens: Aliens,
    bases: Vec<Base>,
    block: Drawable<Rect>,
    ufo: Ufo,
    lives: usize,
    heart: IndexedImage,
    result: GameUpdateResult,
    score: usize,
    #[allow(unused)] //needed to play sound
    audio_engine: AudioEngine,
    controller: GameController,
}

impl PlayerAttack {
    pub fn new(xy: Coord) -> Self {
        Self { xy, next_move: 0.0 }
    }
}

impl Base {
    pub fn new(idx: isize) -> Self {
        let offset = Coord::new(idx * BASE_SPACE + BASE_START.x, BASE_START.y);
        let blocks = vec![vec![true; 6]; 3];
        Self { offset, blocks }
    }
}

impl Aliens {
    pub fn new(
        death_sound: SoundEffect,
        move_sound1: SoundEffect,
        move_sound2: SoundEffect,
    ) -> Self {
        let ships = vec![
            AnimatedIndexedImage::from_file_contents(include_bytes!("../../assets/invader1.ica"))
                .unwrap()
                .0,
            AnimatedIndexedImage::from_file_contents(include_bytes!("../../assets/invader2.ica"))
                .unwrap()
                .0,
            AnimatedIndexedImage::from_file_contents(include_bytes!("../../assets/invader3.ica"))
                .unwrap()
                .0,
        ];
        let attack = AnimatedIndexedImage::from_file_contents(include_bytes!(
            "../../assets/alien_attack.ica"
        ))
        .unwrap()
        .0;
        Self {
            death_sound,
            move_sounds: [move_sound1, move_sound2],
            ships,
            alive: vec![vec![true; ALIEN_ROW_SIZE]; 5],
            offset: ALIEN_START,
            dir: 1,
            last_move: 0.0,
            move_rate: ALIEN_SPEED_START,
            next_move_sound: 0,
            attacks: vec![],
            next_attack: ALIEN_ATTACK_RATE * 2.0,
            attack,
        }
    }
}

impl Ufo {
    pub fn new(sound: SoundEffect, death_sound: SoundEffect) -> Self {
        let mut sprite =
            AnimatedIndexedImage::from_file_contents(include_bytes!("../../assets/ufo.ica"))
                .unwrap()
                .0;
        sprite.set_animate(true);
        Self {
            sprite,
            active_sound: sound,
            death_sound,
            xy: UFO_START,
            next_move: 0.0,
            next_appearance: UFO_RATE,
            is_visible: false,
        }
    }
}

impl Player {
    pub fn new(attack_sound: SoundEffect, death_sound: SoundEffect) -> Self {
        let mut ship = ShapeCollection::default();
        InsertShape::insert_above(
            &mut ship,
            Triangle::equilateral((10, 10), 10, FlatSide::Bottom),
            fill(CLR_SHIP),
        );
        InsertShape::insert_above(&mut ship, Rect::new((0, 10), (20, 16)), fill(CLR_SHIP));
        ship = ship.with_move((70, PLAYER_Y));
        let mut attack = ShapeCollection::default();
        InsertShape::insert_above(&mut attack, Rect::new((0, 0), (1, 6)), fill(CLR_ATTACK));
        Self {
            attack_sound,
            death_sound,
            ship,
            attacks: vec![],
            next_move: 0.0,
            next_attack: Timer::new(PLAYER_ATTACK_RATE),
            attack,
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(
            (self.ship.left(), self.ship.top()),
            (self.ship.right(), self.ship.bottom()),
        )
    }
}

impl Invaders {
    pub fn new() -> Box<Self> {
        let audio_engine = AudioEngine::new().unwrap();
        let player_attack = audio_engine
            .load_from_bytes(include_bytes!("../../assets/player_shoot.wav"), 0.5)
            .unwrap();
        let player_death = audio_engine
            .load_from_bytes(include_bytes!("../../assets/player_dead.wav"), 1.0)
            .unwrap();
        let mut ufo = audio_engine
            .load_from_bytes(include_bytes!("../../assets/ufo.wav"), 1.7)
            .unwrap();
        let ufo_dead = audio_engine
            .load_from_bytes(include_bytes!("../../assets/invader_dead.wav"), 0.5)
            .unwrap();
        let invader_dead = audio_engine
            .load_from_bytes(include_bytes!("../../assets/invader_dead.wav"), 0.5)
            .unwrap();
        let invader_move1 = audio_engine
            .load_from_bytes(include_bytes!("../../assets/invader_move_1.wav"), 0.2)
            .unwrap();
        let invader_move2 = audio_engine
            .load_from_bytes(include_bytes!("../../assets/invader_move_2.wav"), 0.2)
            .unwrap();

        ufo.set_loop(true);

        let block = Drawable::from_obj(Rect::new((0, 0), (4, 4)), fill(CLR_BASE));
        let heart = IndexedImage::from_file_contents(include_bytes!("../../assets/heart.ici"))
            .unwrap()
            .0;

        Box::new(Self {
            audio_engine,
            score: 0,
            result: Nothing,
            aliens: Aliens::new(invader_dead, invader_move1, invader_move2),
            player: Player::new(player_attack, player_death),
            ufo: Ufo::new(ufo, ufo_dead),
            bases: vec![Base::new(0), Base::new(1), Base::new(2)],
            block,
            lives: 3,
            heart,
            controller: GameController::new_unchecked(),
        })
    }

    fn player_killed(&mut self) {
        self.aliens.alive.fill(vec![true; ALIEN_ROW_SIZE]);
        self.aliens.attacks.clear();
        self.aliens.offset = ALIEN_START;
        self.aliens.move_rate = ALIEN_SPEED_START;
        self.aliens.next_attack = ALIEN_ATTACK_RATE;
        self.player.attacks.clear();
        self.ufo.xy = UFO_START;
        self.ufo.next_appearance = UFO_RATE;
        self.ufo.is_visible = false;
        self.ufo.active_sound.reset();
    }

    fn detect_alien_attack_hit(&mut self, timing: &Timing) -> bool {
        let mut remove = vec![];
        for (i, attack) in self.aliens.attacks.iter_mut().enumerate() {
            attack.next_move -= timing.fixed_time_step;
            if attack.next_move <= 0.0 {
                attack.xy = attack.xy + (0, 1);
                attack.next_move = ALIEN_ATTACK_SPEED;
            }
            if attack.xy.y >= SCREEN_HEIGHT as isize {
                remove.push(i);
            }
            if attack.bounds().intersects_rect(&self.player.bounds()) {
                return true;
            }
        }
        for i in remove {
            self.aliens.attacks.remove(i);
        }
        false
    }
}

impl Game for Invaders {
    fn render(&self, graphics: &mut Graphics, controller: Option<Controller>) {
        for i in 1..=self.lives {
            graphics.draw_indexed_image(
                (HEART_XY.x + (HEART_SPACE * (i as isize - 1)), HEART_XY.y),
                &self.heart,
            );
        }

        graphics.draw_text(
            &format!("Score {: >5}", self.score),
            SCORE_XY,
            (CLR_TEXT, TextSize::Normal, RightTop),
        );

        for attack in &self.player.attacks {
            graphics.draw(&self.player.attack.with_move(attack.xy));
        }

        for attack in &self.aliens.attacks {
            graphics.draw_animated_image(attack.xy, &self.aliens.attack);
        }

        graphics.draw(&self.player.ship);
        if self.ufo.is_visible {
            graphics.draw_animated_image(self.ufo.xy, &self.ufo.sprite);
        }

        for (y, row) in self.aliens.alive.iter().enumerate() {
            for (x, ship_alive) in row.iter().enumerate() {
                if *ship_alive {
                    let xy = self.aliens.coord_for_ship(y, x);
                    match y {
                        0 => graphics.draw_animated_image(xy, &self.aliens.ships[0]),
                        1..=2 => graphics.draw_animated_image(xy, &self.aliens.ships[1]),
                        3..=4 => graphics.draw_animated_image(xy, &self.aliens.ships[2]),
                        _ => {}
                    }
                }
            }
        }

        if cfg!(debug_assertions) {
            graphics.draw_text(
                &format!(
                    "{:.2} {}",
                    self.aliens.next_attack,
                    self.aliens.attacks.len(),
                ),
                Px(SCREEN_WIDTH as isize, SCREEN_HEIGHT as isize),
                (CLR_1, TextSize::Small, SpaceBeforeCol(8), RightBottom),
            );
        }
    }

    fn on_key_press(&mut self, _: KeyCode) {}

    #[allow(clippy::collapsible_if)] //for readability
    fn update(
        &mut self,
        timing: &Timing,
        held_keys: &Vec<&KeyCode>,
        controller: &GameController,
    ) -> GameUpdateResult {
        self.controller.update();
        self.ufo.active_sound.update(timing);
        self.ufo.death_sound.update(timing);
        self.player.attack_sound.update(timing);
        self.player.death_sound.update(timing);
        self.aliens.death_sound.update(timing);
        self.aliens.move_sounds[0].update(timing);
        self.aliens.move_sounds[1].update(timing);
        if self.player.next_move < 0.0 {
            if held_keys.contains(&&KeyCode::ArrowLeft) || self.controller.direction.left {
                if self.player.ship.left() > 0 {
                    self.player.ship = self.player.ship.with_translation((-1, 0));
                    self.player.next_move = PLAYER_SPEED;
                }
            } else if held_keys.contains(&&KeyCode::ArrowRight) || self.controller.direction.right {
                if self.player.ship.right() < SCREEN_WIDTH as isize {
                    self.player.ship = self.player.ship.with_translation((1, 0));
                    self.player.next_move = PLAYER_SPEED;
                }
            }
        }

        if held_keys.contains(&&KeyCode::Space) || self.controller.action.south {
            if self.player.attacks.len() < MAX_PLAYER_ATTACKS {
                if self.player.next_attack.update(timing) {
                    self.player
                        .attacks
                        .push(PlayerAttack::new(self.player.ship.center()));
                    self.player.attack_sound.play();
                }
            }
        } else if held_keys.contains(&&KeyCode::Escape) || self.controller.action.east {
            self.result = Pop;
        }

        let mut remove = None;
        for (i, attack) in self.player.attacks.iter_mut().enumerate() {
            if attack.next_move < 0.0 {
                attack.next_move = PLAYER_ATTACK_SPEED;
                attack.xy = attack.xy - (0, 1);
            }
            if attack.xy.y < -3 {
                remove = Some(i);
            }
            attack.next_move -= timing.fixed_time_step;
        }
        if let Some(i) = remove {
            self.player.attacks.remove(i);
        }
        if self.player.attacks.is_empty() {
            self.player.next_attack.trigger()
        }
        if !self.ufo.is_visible && self.ufo.next_appearance < 0.0 {
            self.ufo.xy = Coord::from((SCREEN_WIDTH + 10, 10));
            self.ufo.next_move = UFO_SPEED;
            self.ufo.is_visible = true;
            self.ufo.active_sound.play();
        }
        if self.ufo.is_visible && self.ufo.next_move < 0.0 {
            self.ufo.xy = self.ufo.xy - (1, 0);
            self.ufo.next_move = UFO_SPEED;
            if self.ufo.xy.x < -30 {
                self.ufo.is_visible = false;
                self.ufo.next_appearance = UFO_RATE;
                self.ufo.active_sound.reset();
            }
        }
        self.aliens.update(timing);

        for image in &mut self.aliens.ships {
            image.update(timing.fixed_time_step);
        }

        if self.detect_alien_attack_hit(timing) {
            self.player_killed();
        }

        if let Some(y) = self.aliens.bottom_most_y() {
            if y >= (SCREEN_HEIGHT as isize - 10) {
                self.player_killed();
            }
        }

        if self.aliens.next_attack <= 0.9 && self.aliens.attacks.len() < MAX_ALIENS_ATTACKS {
            if let Some(pos) = self.aliens.random_attack_position() {
                self.aliens.attacks.push(EnemyAttack {
                    xy: pos,
                    next_move: 0.0,
                });
            }
            self.aliens.next_attack = ALIEN_ATTACK_RATE;
        }
        self.aliens.next_attack -= timing.fixed_time_step;

        self.aliens.attack.update(timing.fixed_time_step);

        let mut to_delete = vec![];
        for (i, attack) in self.player.attacks.iter().enumerate() {
            if Rect::new_with_size(
                self.ufo.xy,
                self.ufo.sprite.width() as usize,
                self.ufo.sprite.height() as usize,
            )
            .contains(attack.xy)
            {
                to_delete.push(i);
                self.score += SCORE_UFO;
                self.ufo.death_sound.play();
                self.ufo.is_visible = false;
                self.ufo.next_appearance = UFO_RATE * 2.0;
                self.ufo.active_sound.reset();
            } else if self.aliens.detect_player_attack_hit(attack.xy) {
                to_delete.push(i);
                self.aliens.death_sound.play();
                self.score += SCORE_INVADER;
            }
        }
        for i in to_delete.iter().rev() {
            self.player.attacks.remove(*i);
        }

        self.ufo.next_move -= timing.fixed_time_step;
        self.ufo.next_appearance -= timing.fixed_time_step;
        self.player.next_move -= timing.fixed_time_step;

        self.ufo.sprite.update(timing.fixed_time_step);

        self.result
    }

    fn resuming(&mut self) {}
}
