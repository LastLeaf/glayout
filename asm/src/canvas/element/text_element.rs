use std::rc::Rc;
use super::super::CanvasConfig;
use super::super::character::{Character, FontStyle};
use super::{Element, ElementStyle, InlinePositionStatus};
use super::super::super::tree::{TreeNodeWeak, TreeNodeRc};

const DEFAULT_DPR: f64 = 2.;

// basic text element

pub struct Text {
    tree_node: Option<TreeNodeWeak<Element>>,
    canvas_config: Rc<CanvasConfig>,
    device_pixel_ratio: f64,
    text: String,
    characters: Box<[(Rc<Character>, f32, f32)]>,
    need_update: bool,
    font_family_id: i32,
    tex_font_size: i32,
    size_ratio: f64,
    line_first_char_index: usize,
    line_current_char_index: usize,
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
            line_first_char_index: 0,
            line_current_char_index: 0,
        }
    }
    pub fn set_text<T>(&mut self, s: T) where String: From<T> {
        self.need_update = true;
        self.text = String::from(s);
        let t = self.tree_node.as_mut().unwrap().upgrade().unwrap();
        t.elem().mark_dirty();
    }
    // FIXME update if font_size / font_style / font_family updated

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
        let cm = self.canvas_config.character_manager();
        let mut manager = cm.borrow_mut();
        self.font_family_id = manager.font_family_id(style.font_family.clone());
        self.characters = manager.alloc_chars(self.font_family_id, self.tex_font_size, FontStyle::Normal, self.text.chars());
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
    fn suggest_size(&mut self, suggested_size: (f64, f64), inline_position_status: &mut InlinePositionStatus, style: &ElementStyle) -> (f64, f64) {
        if self.need_update {
            self.update(style);
        }
        let prev_inline_height = inline_position_status.height();
        let line_height = style.font_size; // FIXME use line_height
        let baseline_top = line_height / 2.;
        inline_position_status.append_node(self.tree_node.as_mut().unwrap().upgrade().unwrap(), style.font_size, baseline_top);
        self.line_first_char_index = 0;
        for i in 0..self.characters.len() {
            let v = &mut self.characters[i];
            let character = &v.0;
            if character.tex_id() == -1 {
                if character.unicode_char() == '\n' {
                    inline_position_status.line_wrap();
                    self.line_first_char_index = i;
                }
                self.line_current_char_index = i;
            } else {
                let char_pos = character.position();
                let width = char_pos.4 * self.size_ratio;
                let (left, line_baseline_top) = inline_position_status.add_width(width, true);
                if left == 0. {
                    self.line_first_char_index = i;
                }
                self.line_current_char_index = i;
                v.1 = left as f32;
                v.2 = (line_baseline_top - baseline_top) as f32;
            }
        };
        (suggested_size.0, inline_position_status.height() - prev_inline_height)
    }
    fn adjust_baseline_offset(&mut self, add_offset: f64) {
        for i in self.line_first_char_index..(self.line_current_char_index + 1) {
            self.characters[i].2 += add_offset as f32;
        }
    }
    fn draw(&mut self, _style: &ElementStyle, pos: (f64, f64, f64, f64)) {
        debug!("Attempted to draw Text at {:?}", pos);
        // FIXME whole element edge cutting
        self.characters.iter().for_each(|(character, left, top)| {
            if character.tex_id() == -1 {
                /* empty */
            } else {
                let char_pos = character.position();
                let width = char_pos.4 * self.size_ratio;
                let height = char_pos.5 * self.size_ratio;
                let rm = self.canvas_config.resource_manager();
                rm.borrow_mut().request_draw(
                    character.tex_id(),
                    char_pos.0, char_pos.1, char_pos.2, char_pos.3,
                    pos.0 + *left as f64, pos.1 + *top as f64, width, height
                );
            }
        });
    }
}
