use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::utils::PretendSend;
use super::super::CanvasConfig;
use super::super::character::{CharacterManager, Character, FontStyle};
use super::{ElementStyle, BoundingRect};

// basic text element

pub struct Text {
    canvas_index: i32,
    character_manager: PretendSend<Rc<RefCell<CharacterManager>>>,
    text: String,
    characters: PretendSend<Vec<Rc<Character>>>,
    need_update: bool,
}

impl Text {
    pub fn new(cfg: &CanvasConfig) -> Self {
        Text {
            canvas_index: cfg.index,
            character_manager: PretendSend::new(cfg.get_character_manager()),
            text: String::from(""),
            characters: PretendSend::new(vec![]),
            need_update: true,
        }
    }
    pub fn set_text(&mut self, s: String) {
        self.need_update = true;
        self.text = s;
    }
}

impl super::ElementContent for Text {
    fn name(&self) -> &'static str {
        "Text"
    }
    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect) {
        if self.need_update {
            // NOTE for simplexity, tex generation is delayed to closest animation frame
            let mut manager = self.character_manager.borrow_mut();
            manager.free_chars(&mut self.characters);
            self.characters = PretendSend::new(manager.alloc_chars(0, 16, FontStyle::Normal, self.text.chars()));
        }
        debug!("Attempted to draw Text: {}", self.text);
        self.characters.iter().for_each(|character| {
            lib!(tex_draw(self.canvas_index, 0, character.get_tex_id(), 0., 0., 1., 1., style.left, style.top, style.width, style.height));
            lib!(tex_draw_end(self.canvas_index, 1));
        });
    }
}
