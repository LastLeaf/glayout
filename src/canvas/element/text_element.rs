use std::rc::Rc;
use super::super::CanvasConfig;
use super::super::resource::DrawState;
use super::super::character::{Character, FontStyle};
use super::{Element, ElementStyle, InlineAllocator, Transform, style, Position, Size, Point, Bounds};
use rc_forest::ForestNode;

const DEFAULT_DPR: f64 = 2.;

// basic text element

pub struct Text {
    element: *mut Element,
    canvas_config: Rc<CanvasConfig>,
    device_pixel_ratio: f64,
    text: String,
    characters: Box<[(Rc<Character>, f32, f32)]>,
    need_update: bool,
    font_family_id: i32,
    tex_font_size: i32,
    size_ratio: f32,
    line_first_char_index: usize,
    line_current_char_index: usize,
    drawing_bounds: Bounds,
}

impl Text {
    pub fn new(cfg: &Rc<CanvasConfig>) -> Self {
        Self {
            element: 0 as *mut Element,
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
            drawing_bounds: Bounds::new(0., 0., 0., 0.),
        }
    }
    #[inline]
    fn element<'a>(&'a self) -> &'a Element {
        unsafe { &*self.element }
    }
    #[inline]
    fn node<'a>(&'a self) -> &'a ForestNode<Element> {
        self.element().node()
    }
    #[inline]
    fn element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    fn node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self.element_mut().node_mut()
    }

    pub fn set_text<T>(&mut self, s: T) where String: From<T> {
        self.need_update = true;
        self.text = String::from(s);
        self.element_mut().mark_layout_dirty();
    }
    pub fn get_text(&mut self) -> String {
        self.text.clone()
    }

    // FIXME update if font_style updated
    fn check_font_changed(&mut self, style: &ElementStyle) {
        let font_size = style.get_font_size();
        if self.tex_font_size != self.measure_tex_font_size(font_size) {
            self.need_update = true;
            return;
        }
        let cm = self.canvas_config.character_manager();
        let mut manager = cm.borrow_mut();
        if self.font_family_id != manager.font_family_id(style.get_font_family().clone()) {
            self.need_update = true;
            return;
        }
    }
    fn measure_tex_font_size(&mut self, font_size: f32) -> i32 {
        let min_font_size = (font_size * self.device_pixel_ratio as f32).ceil();
        min_font_size as i32
    }
    fn update(&mut self, style: &ElementStyle) {
        self.need_update = false;
        // FIXME consider batching multiple text element update together
        let font_size = style.get_font_size();
        self.tex_font_size = self.measure_tex_font_size(font_size);
        self.size_ratio = font_size / self.tex_font_size as f32;
        // debug!("Attempted to regenerate Text: \"{}\" font {:?} size {:?}", self.text, style.get_font_family(), self.tex_font_size);
        let cm = self.canvas_config.character_manager();
        let mut manager = cm.borrow_mut();
        self.font_family_id = manager.font_family_id(style.get_font_family().clone());
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
    fn clone(&self) -> Box<super::ElementContent> {
        let cfg = &self.canvas_config;
        Box::new(Self {
            element: 0 as *mut Element,
            canvas_config: cfg.clone(),
            device_pixel_ratio: if cfg.device_pixel_ratio == 1. { DEFAULT_DPR } else { cfg.device_pixel_ratio },
            text: self.text.clone(),
            characters: self.characters.clone(),
            need_update: false,
            tex_font_size: self.tex_font_size,
            font_family_id: self.font_family_id,
            size_ratio: self.size_ratio,
            line_first_char_index: 0,
            line_current_char_index: 0,
            drawing_bounds: Bounds::new(0., 0., 0., 0.),
        })
    }
    #[inline]
    fn associate_element(&mut self, element: *mut Element) {
        self.element = element;
    }
    fn suggest_size(&mut self, suggested_size: Size, inline_allocator: &mut InlineAllocator, style: &ElementStyle) -> Size {
        self.check_font_changed(style);
        if self.need_update {
            self.update(style);
        }
        let base_requested_top = inline_allocator.get_current_height();
        let initial_line_top = -inline_allocator.get_current_line_height();
        let line_height = if style.get_line_height() == style::DEFAULT_F32 { style.get_font_size() * 1.5 } else { style.get_line_height() };
        let character_baseline_top = line_height / 2.;
        inline_allocator.start_node(self.node_mut(), line_height as f64, character_baseline_top as f64);
        self.line_first_char_index = 0;
        for i in 0..self.characters.len() {
            let character = &self.characters[i].0.clone();
            if character.tex_id() == -1 {
                if character.unicode_char() == '\n' {
                    inline_allocator.line_wrap(self.node_mut());
                    self.line_first_char_index = i;
                }
                self.line_current_char_index = i;
            } else {
                let char_pos = character.position();
                let width = char_pos.4 * self.size_ratio as f64;
                let (left, line_baseline_top) = inline_allocator.add_width(self.node_mut(), width, true).into();
                if left == 0. {
                    self.line_first_char_index = i;
                }
                self.line_current_char_index = i;
                let v = &mut self.characters[i];
                v.1 = left as f32;
                v.2 = line_baseline_top as f32 - character_baseline_top - base_requested_top as f32;
            }
        };
        self.drawing_bounds = Bounds::new(0., initial_line_top, suggested_size.width(), inline_allocator.get_current_height() - base_requested_top);
        Size::new(suggested_size.width(), inline_allocator.get_current_height() - base_requested_top)
    }
    fn adjust_baseline_offset(&mut self, add_offset: f64) {
        for i in self.line_first_char_index..(self.line_current_char_index + 1) {
            self.characters[i].2 += add_offset as f32;
        }
        self.drawing_bounds.extend_bottom(add_offset);
    }
    fn adjust_text_align_offset(&mut self, add_offset: f64) {
        for i in self.line_first_char_index..(self.line_current_char_index + 1) {
            self.characters[i].1 += add_offset as f32;
        }
    }
    fn draw(&mut self, transform: &Transform) {
        // debug!("Attempted to draw Text at {:?}", transform.apply_to_position(&(0., 0., 0., 0.)));
        // FIXME whole element edge cutting
        for (character, left, top) in self.characters.iter() {
            if character.tex_id() == -1 {
                /* empty */
            } else {
                let char_pos = character.position();
                let width = char_pos.4 * self.size_ratio as f64;
                let height = char_pos.5 * self.size_ratio as f64;
                let rm = self.canvas_config.resource_manager();
                let mut rm = rm.borrow_mut();
                rm.set_draw_state(DrawState::new().color(self.element().style().get_color()));
                rm.request_draw(
                    character.tex_id(), true,
                    char_pos.0, char_pos.1, char_pos.2, char_pos.3,
                    transform.apply_to_position(&Position::new(*left as f64, *top as f64, width, height)).into()
                );
            }
        }
    }
    #[inline]
    fn drawing_bounds(&self) -> Bounds {
        self.drawing_bounds
    }
    fn is_under_point(&self, point: Point, transform: Transform) -> bool {
        // FIXME use area detection
        for (character, left, top) in self.characters.iter() {
            if character.tex_id() == -1 {
                /* empty */
            } else {
                let char_pos = character.position();
                let width = char_pos.4 * self.size_ratio as f64;
                let height = char_pos.5 * self.size_ratio as f64;
                let pos = transform.apply_to_position(&Position::new(*left as f64, *top as f64, width, height));
                // debug!("testing {:?} in text pos {:?}", (x, y), pos);
                if !point.in_position(&pos) {
                    continue;
                }
                return true
            }
        }
        false
    }
}
