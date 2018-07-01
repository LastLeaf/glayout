use std::cmp;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::str::Chars;
use std::ffi::CString;
use super::super::utils::PretendSend;
use super::resource::ResourceManager;
use std::collections::HashMap;

const MAX_TEX_SIZE: i32 = 4096;
const MIN_FONT_SIZE: i32 = 1;

lazy_static! {
    static ref FONT_FAMILY_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
    static ref FONT_FAMILY_MAP: PretendSend<RefCell<HashMap<String, i32>>> = PretendSend::new(RefCell::new(HashMap::new()));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(PartialEq, Eq, Hash)]
struct CharacterKey {
    pub unicode: char,
    pub font_family_id: i32,
    pub font_size: i32,
    pub font_style: FontStyle,
}

#[derive(Debug)]
pub struct Character {
    unicode: char,
    font_family_id: i32,
    font_style: FontStyle,
    font_size: f64,
    left: Cell<f64>,
    top: Cell<f64>,
    width: Cell<f64>,
    height: Cell<f64>,
    natural_width: Cell<f64>,
    natural_height: Cell<f64>,
    tex_id: Cell<i32>,
}

impl Character {
    pub fn new(unicode: char, font_family_id: i32, font_size: i32, font_style: FontStyle) -> Self {
        Self {
            unicode,
            font_family_id,
            font_size: font_size as f64,
            font_style,
            left: Cell::new(0.),
            top: Cell::new(0.),
            width: Cell::new(0.),
            height: Cell::new(0.),
            natural_width: Cell::new(0.),
            natural_height: Cell::new(0.),
            tex_id: Cell::new(-1),
        }
    }

    #[inline]
    pub fn get_char(&self) -> char {
        self.unicode
    }
    #[inline]
    pub fn get_font_size(&self) -> f64 {
        self.font_size
    }
    #[inline]
    fn set_position(&self, left: f64, top: f64, width: f64, height: f64, natural_width: f64, natural_height: f64) {
        self.left.set(left);
        self.top.set(top);
        self.width.set(width);
        self.height.set(height);
        self.natural_width.set(natural_width);
        self.natural_height.set(natural_height);
    }
    #[inline]
    fn normalize_size(&self, left_offset: f64, top_offset: f64, total_width: f64, total_height: f64) {
        self.left.set((self.left.get() + left_offset) / total_width);
        self.top.set((self.top.get() + top_offset) / total_height);
        self.width.set(self.width.get() / total_width);
        self.height.set(self.height.get() / total_height);
    }
    #[inline]
    pub fn get_position(&self) -> (f64, f64, f64, f64, f64, f64) {
        (self.left.get(), self.top.get(), self.width.get(), self.height.get(), self.natural_width.get(), self.natural_height.get())
    }
    #[inline]
    fn set_tex_id(&self, tex_id: i32) {
        self.tex_id.set(tex_id);
    }
    #[inline]
    pub fn get_tex_id(&self) -> i32 {
        self.tex_id.get()
    }
}

#[inline]
fn get_default_line_height(font_size: i32) -> f64 {
    (font_size as f64 * 1.5).ceil()
}

pub struct CharacterManager {
    canvas_index: i32,
    resource_manager: PretendSend<Rc<RefCell<ResourceManager>>>,
    char_tex_id_map: HashMap<CharacterKey, Rc<Character>>,
    tex_allocator: CharacterTexAllocator,
}

fn draw_to_tex(canvas_index: i32, tex_allocator: &mut CharacterTexAllocator, characters: &mut Vec<Rc<Character>>, font_size: i32) {
    let mut left: f64 = 0.;
    let mut top: f64 = 0.;
    let mut total_width: f64 = 0.;
    let line_height = get_default_line_height(font_size);
    let mut string_to_draw = String::new();
    characters.iter().for_each(|character| {
        let mut s = String::new();
        s.push(character.unicode);
        let width = lib!(text_get_width(CString::new(s).unwrap().into_raw())); // FIXME should be able to batch
        if left + width >= MAX_TEX_SIZE as f64 {
            total_width = if total_width > left { total_width } else { left };
            left = 0.;
            top += line_height;
            string_to_draw.push('\n');
        }
        string_to_draw.push(character.unicode);
        character.set_position(left, top, width, line_height, width, line_height);
        left += width;
    });
    total_width = if total_width > left { total_width } else { left };
    total_width = total_width.ceil();
    let total_height = if left > 0. { top + line_height } else { top };
    let (tex_id, left, top) = tex_allocator.alloc_tex_pos(total_width as i32, total_height as i32);
    lib!(text_to_tex(canvas_index, tex_id, left, top, CString::new(string_to_draw).unwrap().into_raw(), total_width as i32, total_height as i32, line_height as i32));
    characters.iter().for_each(|character| {
        character.normalize_size(left as f64, top as f64, MAX_TEX_SIZE as f64, MAX_TEX_SIZE as f64);
        character.set_tex_id(tex_id);
    });
}

impl CharacterManager {
    pub fn new(canvas_index: i32, resource_manager: Rc<RefCell<ResourceManager>>) -> Self {
        Self {
            canvas_index,
            resource_manager: PretendSend::new(resource_manager.clone()),
            char_tex_id_map: HashMap::new(),
            tex_allocator: CharacterTexAllocator::new(canvas_index, resource_manager),
        }
    }

    pub fn alloc_chars(&mut self, font_family_id: i32, font_size: i32, font_style: FontStyle, chars: Chars) -> Box<[Rc<Character>]> {
        let font_size = cmp::max(font_size, MIN_FONT_SIZE);
        let line_height = get_default_line_height(font_size);
        let tex_batch_max = (MAX_TEX_SIZE / (font_size * 2)) * (MAX_TEX_SIZE / line_height as i32);
        lib!(text_set_font(font_size, line_height as i32, font_family_id, (font_style == FontStyle::Italic || font_style == FontStyle::BoldItalic) as i32, (font_style == FontStyle::Bold || font_style == FontStyle::BoldItalic) as i32));
        let mut characters_to_draw: Vec<Rc<Character>> = vec!();
        let mut characters_to_draw_count = 0;
        let characters = chars.map(|c| {
            let mut need_insert = false;
            let mut key = CharacterKey {
                unicode: c,
                font_family_id,
                font_size,
                font_style,
            };
            let character = match self.char_tex_id_map.get(&mut key) {
                Some(x) => {
                    x.clone()
                },
                None => {
                    let character = Rc::new(Character::new(c, font_family_id, font_size, font_style));
                    if c < ' ' {
                        character.set_position(0., 0., 0., 0., 0., line_height);
                    } else {
                        characters_to_draw.push(character.clone());
                        characters_to_draw_count += 1;
                        if characters_to_draw_count == tex_batch_max {
                            draw_to_tex(self.canvas_index, &mut self.tex_allocator, &mut characters_to_draw, font_size);
                            characters_to_draw.truncate(0);
                            characters_to_draw_count = 0;
                        }
                        need_insert = true;
                    }
                    character
                }
            };
            if need_insert {
                self.char_tex_id_map.insert(key, character.clone());
            }
            character
        }).collect::<Vec<Rc<Character>>>().into_boxed_slice();
        if characters_to_draw.len() > 0 {
            draw_to_tex(self.canvas_index, &mut self.tex_allocator, &mut characters_to_draw, font_size);
        }
        characters
    }

    fn _gabbage_collect() {
        // TODO gabbage collect chars when needed
    }

    fn alloc_font_family_id() -> i32 {
        let ret = FONT_FAMILY_ID_INC.get();
        FONT_FAMILY_ID_INC.set(ret + 1);
        ret
    }
    pub fn get_font_family_id(&mut self, name: String) -> i32 {
        // NOTE font-family is never released
        let mut font_family_map = FONT_FAMILY_MAP.borrow_mut();
        let mut need_insert = false;
        let font_family_id = match font_family_map.get(&name) {
            Some(x) => {
                *x
            },
            None => {
                let font_family_id = Self::alloc_font_family_id();
                lib!(text_bind_font_family(font_family_id, CString::new(name.clone()).unwrap().into_raw()));
                need_insert = true;
                font_family_id
            }
        };
        if need_insert {
            font_family_map.insert(name, font_family_id);
        }
        font_family_id
    }
}

struct CharacterTexAllocator {
    canvas_index: i32,
    resource_manager: Rc<RefCell<ResourceManager>>,
    filled_height: i32,
    half_filled_height: i32,
    half_filled_left: i32,
    tex_ids: Vec<i32>,
}

impl CharacterTexAllocator {
    fn new(canvas_index: i32, rm: Rc<RefCell<ResourceManager>>) -> Self {
        Self {
            canvas_index,
            resource_manager: rm,
            filled_height: MAX_TEX_SIZE,
            half_filled_height: MAX_TEX_SIZE,
            half_filled_left: 0,
            tex_ids: vec![],
        }
    }

    fn alloc_tex_pos(&mut self, width: i32, height: i32) -> (i32, i32, i32) {
        let mut use_half_filled = self.half_filled_left + width <= MAX_TEX_SIZE;
        let top = if use_half_filled { self.half_filled_height } else { self.filled_height };
        if top + height > MAX_TEX_SIZE {
            let new_tex_id = self.resource_manager.borrow_mut().alloc_tex_id();
            lib!(tex_create_empty(self.canvas_index, new_tex_id, MAX_TEX_SIZE, MAX_TEX_SIZE));
            self.tex_ids.push(new_tex_id);
            self.filled_height = 0;
            self.half_filled_height = 0;
            self.half_filled_left = 0;
            use_half_filled = false;
        }
        let tex_id = self.tex_ids[self.tex_ids.len() - 1];
        if use_half_filled {
            let ret = (tex_id, self.half_filled_left, self.half_filled_height);
            if self.filled_height < self.half_filled_height + height {
                self.filled_height = self.half_filled_height + height
            }
            self.half_filled_left += width;
            ret
        } else {
            let ret = (tex_id, 0, self.filled_height);
            if self.half_filled_left > width {
                self.half_filled_height = self.filled_height;
            }
            self.filled_height += height;
            self.half_filled_left = width;
            ret
        }
    }
}
