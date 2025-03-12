use log::error;
use pixels_graphics_lib::prelude::PixelFont::Standard4x5;
use pixels_graphics_lib::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::OnceLock;

pub const BAR_HEIGHT: usize = 20;
pub const ICON_SIZE: (usize, usize) = (17, 17);
pub const ICON_PADDING: usize = 2;

static ICONS: OnceLock<HashMap<ButtonDef, HashMap<Option<Controller>, IndexedImage>>> =
    OnceLock::new();

fn get_icon(def: ButtonDef, controller: Option<Controller>) -> &'static IndexedImage {
    &ICONS.get_or_init(|| {
        HashMap::from([
            (
                ButtonDef::Space,
                HashMap::from([
                    (
                        Some(Controller::Playstation),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_x.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Xbox),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_a.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Switch),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_b.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        None,
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/key_space.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                ]),
            ),
            (
                ButtonDef::Escape,
                HashMap::from([
                    (
                        Some(Controller::Playstation),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_o.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Xbox),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_b.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Switch),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_a.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        None,
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/key_esc.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                ]),
            ),
            (
                ButtonDef::Vert,
                HashMap::from([
                    (
                        Some(Controller::Playstation),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_vert.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Xbox),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_vert.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Switch),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_vert.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        None,
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/key_vert.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                ]),
            ),
            (
                ButtonDef::Horz,
                HashMap::from([
                    (
                        Some(Controller::Playstation),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_horz.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Xbox),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_horz.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Switch),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_horz.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        None,
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/key_horz.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                ]),
            ),
            (
                ButtonDef::Cursor,
                HashMap::from([
                    (
                        Some(Controller::Playstation),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_dpad.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Xbox),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_dpad.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        Some(Controller::Switch),
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/cntr_dpad.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                    (
                        None,
                        IndexedImage::from_file_contents(include_bytes!(
                            "../assets/icons/key_dpad.ici"
                        ))
                        .unwrap()
                        .0,
                    ),
                ]),
            ),
        ])
    })[&def][&controller]
}

pub struct ButtonBar {
    width: usize,
    position: Coord,
    buttons: Vec<(&'static str, ButtonDef, Coord)>,
}

impl ButtonBar {
    pub fn new_blank(position: Coord, width: usize) -> Self {
        ButtonBar {
            width,
            position: position + (0, 2),
            buttons: vec![],
        }
    }

    pub fn new(position: Coord, width: usize, buttons: &[(&'static str, ButtonDef)]) -> Self {
        let mut bar = ButtonBar::new_blank(position, width);
        bar.set_buttons(buttons);
        bar
    }
}

impl ButtonBar {
    pub fn set_buttons(&mut self, buttons: &[(&'static str, ButtonDef)]) {
        self.buttons.clear();
        let widths: Vec<usize> = buttons
            .iter()
            .map(|(text, _)| Standard4x5.measure(text).0 + ICON_SIZE.0 + ICON_PADDING)
            .collect();
        let needed_width: usize = widths.iter().sum::<usize>() + (buttons.len() * 2);
        if needed_width > self.width {
            error!("Buttons are too big: {needed_width} > {}", self.width);
            return;
        }
        let section_width = self.width / buttons.len();
        for (i, (label, def)) in buttons.iter().enumerate() {
            let padding = (section_width - widths[i]) / 2;
            let x = section_width * i;
            self.buttons.push((label, *def, coord!(x + padding, 0)));
        }
    }
}

impl ButtonBar {
    pub fn render(&self, graphics: &mut Graphics, active_controller: Option<Controller>) {
        graphics.with_translate(self.position, |g| {
            for (name, def, pos) in &self.buttons {
                g.draw_indexed_image(pos, get_icon(*def, active_controller));
                g.draw_text(
                    name,
                    TextPos::px(*pos + (ICON_SIZE.0 + ICON_PADDING, 6)),
                    (GB_3, Standard4x5),
                );
            }
        });
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ButtonDef {
    Escape,
    Space,
    Horz,
    Vert,
    Cursor,
}
