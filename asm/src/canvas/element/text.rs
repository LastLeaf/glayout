use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::utils::PretendSend;
use super::super::CanvasConfig;
use super::super::character::{CharacterManager, Character, FontStyle, CACHE_TEX_SIZE};
use super::{ElementStyle, BoundingRect};

// basic text element

pub struct Text {
    canvas_index: i32,
    device_pixel_ratio: f64,
    character_manager: PretendSend<Rc<RefCell<CharacterManager>>>,
    text: String,
    characters: PretendSend<Box<[Rc<Character>]>>,
    need_update: bool,
    font_family_id: i32,
    tex_font_size: i32,
    size_ratio: f64,
}

impl Text {
    pub fn new(cfg: &CanvasConfig) -> Self {
        Text {
            canvas_index: cfg.index,
            device_pixel_ratio: cfg.device_pixel_ratio,
            character_manager: PretendSend::new(cfg.get_character_manager()),
            text: String::from(""),
            characters: PretendSend::new(Box::new([])),
            need_update: true,
            tex_font_size: 0,
            font_family_id: 0,
            size_ratio: 1.,
        }
    }
    pub fn set_text<T>(&mut self, s: T) where String: From<T> {
        self.need_update = true;
        self.text = String::from(s);
    }
    // TODO update if font_size / font_style / font_family updated

    fn generate_tex_font_size(&mut self, font_size: f64) { // TODO do not update if not changed
        let min_font_size = (font_size * self.device_pixel_ratio).ceil();
        self.tex_font_size = min_font_size as i32;
        self.size_ratio = font_size / (self.tex_font_size as f64);
    }
}

impl super::ElementContent for Text {
    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect) {
        if self.need_update {
            self.generate_tex_font_size(style.font_size);
            // debug!("Attempted to regenerate Text: \"{}\" font {} size {}", self.text, style.font_family.clone(), self.tex_font_size);
            // NOTE for simplexity, tex generation is delayed to closest animation frame
            let mut manager = self.character_manager.borrow_mut();
            manager.free_chars(&mut self.characters);
            self.font_family_id = manager.get_font_family_id(style.font_family.clone()); // TODO do not update if not changed
            self.characters = PretendSend::new(manager.alloc_chars(self.font_family_id, self.tex_font_size, FontStyle::Normal, self.text.chars()));
            self.need_update = false;
        }
        // debug!("Attempted to draw Text: {}", self.text);
        let mut left = style.left;
        self.characters.iter().for_each(|character| {
            let pos = character.get_position();
            lib!(tex_draw(self.canvas_index, 0, character.get_tex_id(),
                pos.0 / CACHE_TEX_SIZE as f64, pos.1 / CACHE_TEX_SIZE as f64, 1., 1.,
                left, 0., pos.2, pos.3
            ));
            lib!(tex_draw_end(self.canvas_index, 1));
            left += pos.2;
        });
    }
}
