use std::rc::Rc;
use super::super::CanvasConfig;
use super::super::character::{Character, FontStyle};
use super::{Element, ElementStyle};
use super::super::super::tree::{TreeNodeWeak, TreeNodeRc};

const DEFAULT_DPR: f64 = 2.;

// basic text element

pub struct Text {
    tree_node: Option<TreeNodeWeak<Element>>,
    canvas_config: Rc<CanvasConfig>,
    device_pixel_ratio: f64,
    text: String,
    characters: Box<[Rc<Character>]>,
    need_update: bool,
    font_family_id: i32,
    tex_font_size: i32,
    size_ratio: f64,
}

impl Text {
    pub fn new(cfg: &Rc<CanvasConfig>) -> Self {
        Self {
            tree_node: None,
            canvas_config: cfg.clone(),
            device_pixel_ratio: if cfg.device_pixel_ratio == 1. { DEFAULT_DPR } else { cfg.device_pixel_ratio },
            text: String::from(""),
            characters: Box::new([]),
            need_update: false,
            tex_font_size: 0,
            font_family_id: 0,
            size_ratio: 1.,
        }
    }
    pub fn set_text<T>(&mut self, s: T) where String: From<T> {
        self.need_update = true;
        self.text = String::from(s);
        let t = self.tree_node.as_mut().unwrap().upgrade().unwrap();
        t.elem().mark_dirty();
    }
    // TODO update if font_size / font_style / font_family updated

    fn generate_tex_font_size(&mut self, font_size: f64) {
        let min_font_size = (font_size * self.device_pixel_ratio).ceil();
        self.tex_font_size = min_font_size as i32;
        self.size_ratio = font_size / (self.tex_font_size as f64);
    }
    fn update(&mut self, style: &ElementStyle) {
        self.need_update = false;
        // FIXME consider batching multiple text element update together
        self.generate_tex_font_size(style.font_size);
        // debug!("Attempted to regenerate Text: \"{}\" font {} size {}", self.text, style.font_family.clone(), self.tex_font_size);
        let cm = self.canvas_config.get_character_manager();
        let mut manager = cm.borrow_mut();
        self.font_family_id = manager.get_font_family_id(style.font_family.clone());
        self.characters = manager.alloc_chars(self.font_family_id, self.tex_font_size, FontStyle::Normal, self.text.chars());
        self.need_update = false;
    }
}

impl super::ElementContent for Text {
    #[inline]
    fn name(&self) -> &'static str {
        "Text"
    }
    #[inline]
    fn is_terminated(&self) -> bool {
        true
    }
    #[inline]
    fn associate_tree_node(&mut self, tree_node: TreeNodeRc<Element>) {
        self.tree_node = Some(tree_node.downgrade());
    }
    fn suggest_size(&mut self, suggested_size: (f64, f64), style: &ElementStyle) -> (f64, f64) {
        if self.need_update {
            self.update(style);
        }
        let mut left = 0.;
        let mut top = 0.;
        self.characters.iter().for_each(|character| {
            if character.get_tex_id() == -1 {
                if character.get_char() == '\n' {
                    top += character.get_position().5 * self.size_ratio; // TODO use line height
                    left = 0.;
                }
            } else {
                let char_pos = character.get_position();
                let width = char_pos.4 * self.size_ratio;
                if left + width > suggested_size.0 {
                    top += style.font_size; // TODO use line height
                    left = 0.;
                }
                left += width;
            }
        });
        top += style.font_size; // TODO use line height
        (suggested_size.0, top)
    }
    fn draw(&mut self, style: &ElementStyle, pos: (f64, f64, f64, f64)) {
        debug!("Attempted to draw Text at {:?}", pos);
        // TODO whole element edge cutting
        let mut left = pos.0;
        let mut top = pos.1;
        self.characters.iter().for_each(|character| {
            if character.get_tex_id() == -1 {
                if character.get_char() == '\n' {
                    top += character.get_position().5 * self.size_ratio; // TODO use line height
                    left = 0.;
                }
            } else {
                let char_pos = character.get_position();
                let width = char_pos.4 * self.size_ratio;
                let height = char_pos.5 * self.size_ratio;
                if left + width > pos.2 {
                    top += style.font_size; // TODO use line-height
                    left = 0.;
                }
                let rm = self.canvas_config.get_resource_manager();
                rm.borrow_mut().request_draw(
                    character.get_tex_id(),
                    char_pos.0, char_pos.1, char_pos.2, char_pos.3,
                    left, top, width, height
                );
                left += width;
            }
        });
    }
}
