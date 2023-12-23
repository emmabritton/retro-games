use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use fnv::FnvHashMap;
use pixels_graphics_lib::buffer_graphics_lib::Graphics;
use pixels_graphics_lib::buffer_graphics_lib::prelude::{IndexedImage, TextPos, TextSize, WrappingStrategy};
use pixels_graphics_lib::buffer_graphics_lib::color::WHITE;
use pixels_graphics_lib::buffer_graphics_lib::prelude::TextSize::Normal;
use pixels_graphics_lib::graphics_shapes::coord;
use crate::common::Language;
use crate::common::text::LangText;
use crate::{HEIGHT, WIDTH};

const ICON_OFFSET: usize = 20;

const ICON_ESCAPE = HashMap

pub struct ButtonBarSwapper<T: Hash + Copy + Eq> {
    bars: FnvHashMap<T, ButtonBar>,
    active: T,
}

impl<T: Hash + Copy + Eq> ButtonBarSwapper<T> {
    pub fn new(key: T, bar: ButtonBar) -> ButtonBarSwapper<T> {
        let mut bars = FnvHashMap::default();
        bars.insert(key, bar);
        Self {
            bars,
            active: key,
        }
    }

    pub fn new_builder(lang: Language, key: T, builder: fn(&mut ButtonBar)) -> ButtonBarSwapper<T> {
        let mut bars = FnvHashMap::default();
        let mut bar = ButtonBar::new(lang);
        builder(&mut bar);
        bars.insert(key, bar);
        Self {
            bars,
            active: key,
        }
    }
}

impl<T: Hash + Copy + Eq> ButtonBarSwapper<T> {
    pub fn add_bar(&mut self, key: T, bar: ButtonBar) {
        self.bars.insert(key, bar);
    }

    pub fn set_active(&mut self, key: T) {
        self.active = key;
    }

    pub fn render(&self, graphics: &mut Graphics) {
        self.bars[&self.active].render(graphics)
    }

    pub fn new_bar(&mut self, key: T, setup: fn(&mut ButtonBar)) {
        let mut new_bar = ButtonBar::from(&self.bars[&self.active]);
        setup(&mut new_bar);
        self.bars.insert(key, new_bar);
    }

    pub fn active(&self) -> &T {
        &self.active
    }
}

pub struct Button {
    id: u64,
    label: String,
    def: ButtonDef,
    icon: FnvHashMap<Option<>>
}

pub struct ButtonBar {
    buttons: Vec<(Button, &'static str)>,
    layouts: Vec<(usize, WrappingStrategy)>,
    icons: Rc<FnvHashMap<Button, IndexedImage>>,
}

impl ButtonBar {
    pub fn new(language: Language) -> Self {
        let mut icons = FnvHashMap::default();
        icons.insert(Button::Escape, IndexedImage::from_file_contents(include_bytes!("../../assets/icons/esc.ici")).unwrap().0);
        icons.insert(Button::Arrows, IndexedImage::from_file_contents(include_bytes!("../../assets/icons/arrows.ici")).unwrap().0);
        icons.insert(Button::Return, IndexedImage::from_file_contents(include_bytes!("../../assets/icons/ret.ici")).unwrap().0);
        Self { language, buttons: vec![], layouts: vec![], icons: Rc::new(icons) }
    }

    pub fn from(other: &ButtonBar) -> Self {
        Self {
            language: other.language,
            buttons: vec![],
            layouts: vec![],
            icons: other.icons.clone(),
        }
    }
}

impl ButtonBar {
    fn layout(&mut self) {
        self.layouts.clear();
        let sizes: Vec<usize> = self.buttons.iter().map(|(_, text)| TextSize::measure(&Normal, text, WrappingStrategy::None)).map(|(w, _)| w + ICON_OFFSET).collect();
        let total: usize = sizes.iter().sum();
        let spacing = (WIDTH - total) / sizes.len();
        if total <= WIDTH {
            self.layouts = (0..sizes.len()).map(|i| i * spacing + sizes[0..i].iter().sum::<usize>() + spacing / 2).map(|x| (x, WrappingStrategy::None)).collect();
        } else {
            //truncated
        }
    }
}

impl ButtonBar {
    pub fn clear(&mut self) {
        self.buttons.clear();
    }

    pub fn add(&mut self, button: Button, text: &'static LangText) {
        self.buttons.push((button, &text[self.language]));
        self.layout();
    }

    pub fn render(&self, graphics: &mut Graphics) {
        let y = HEIGHT - 20;
        for (i, (button, text)) in self.buttons.iter().enumerate() {
            let (x, strat) = self.layouts[i];
            let pos = coord!(x,y);
            graphics.draw_indexed_image(pos, &self.icons[button]);
            graphics.draw_text(text, TextPos::px(pos + (ICON_OFFSET, 4)), (WHITE, Normal, strat));
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ButtonDef {
    Escape,
    Space,
    Horz,
    Vert
}
