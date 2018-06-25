use std::cmp;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::str::Chars;
use std::ffi::CString;
use super::super::utils::PretendSend;
use super::resource::ResourceManager;
use std::collections::HashMap;

pub const CACHE_TEX_SIZE: i32 = 4096;
const BG_CANVAS_SIZE: i32 = 4096;
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
            tex_id: Cell::new(-1),
        }
    }

    #[inline]
    pub fn get_font_size(&self) -> f64 {
        self.font_size
    }
    #[inline]
    fn set_position(&self, left: f64, top: f64, width: f64) {
        self.left.set(left);
        self.top.set(top);
        self.width.set(width);
    }
    #[inline]
    pub fn get_position(&self) -> (f64, f64, f64, f64) {
        (self.left.get(), self.top.get(), self.width.get(), self.font_size)
    }
    #[inline]
    fn alloc_tex(&self, tex_id: i32) {
        self.tex_id.set(tex_id);
    }
    #[inline]
    fn free_tex(&self) {
        unimplemented!();
    }
    #[inline]
    pub fn get_tex_id(&self) -> i32 {
        self.tex_id.get()
    }
}

pub struct CharacterManager {
    canvas_index: i32,
    resource_manager: PretendSend<Rc<RefCell<ResourceManager>>>,
    char_tex_id_map: HashMap<CharacterKey, Rc<Character>>,
}

impl CharacterManager {
    pub fn new(canvas_index: i32, resource_manager: Rc<RefCell<ResourceManager>>) -> Self {
        Self {
            canvas_index,
            resource_manager: PretendSend::new(resource_manager),
            char_tex_id_map: HashMap::new(),
        }
    }

    fn draw_to_tex(&self, characters: &mut Vec<Rc<Character>>, whole_string: String, font_size: i32) {
        let mut left = 0.;
        characters.iter().for_each(|character| {
            let mut s = String::new();
            s.push(character.unicode);
            let width = lib!(text_get_width(CString::new(s).unwrap().into_raw())); // FIXME should be able to batch
            character.set_position(left, 0., width);
            let tex_id = self.resource_manager.borrow_mut().alloc_tex_id();
            character.alloc_tex(tex_id);
            lib!(tex_create_empty(self.canvas_index, tex_id, width.ceil() as i32, font_size));
            left += width;
        });
        let total_width = left.ceil() as i32;
        lib!(text_to_tex(self.canvas_index, CString::new(whole_string).unwrap().into_raw(), total_width, font_size));
        // lib!(tex_bind_rendering_target(self.canvas_index, -1, total_width, font_size));
        characters.iter().for_each(|character| {
            let pos = character.get_position();
            let target_width = pos.2.ceil() as i32;
            let target_height = pos.3.ceil() as i32;
            lib!(tex_bind_rendering_target(self.canvas_index, character.get_tex_id(), target_width, target_height));
            lib!(tex_draw(self.canvas_index, 0, -1, pos.0 / total_width as f64, 1., pos.2 / total_width as f64, -1., 0., 0., pos.2, pos.3));
            lib!(tex_draw_end(self.canvas_index, 1));
            // lib!(tex_copy(self.canvas_index, character.get_tex_id(), 0, 0, left.ceil() as i32, 0, target_width, target_height));
            character.set_position(0., 0., pos.2);
        });
        lib!(tex_unbind_rendering_target(self.canvas_index));
    }

    pub fn alloc_chars(&mut self, font_family_id: i32, font_size: i32, font_style: FontStyle, chars: Chars) -> Box<[Rc<Character>]> {
        let font_size = cmp::max(font_size, MIN_FONT_SIZE);
        lib!(text_set_font(font_size, font_family_id, (font_style == FontStyle::Italic || font_style == FontStyle::BoldItalic) as i32, (font_style == FontStyle::Bold || font_style == FontStyle::BoldItalic) as i32));
        let batch_draws_count: usize = (BG_CANVAS_SIZE / (font_size * 2)) as usize;
        let mut characters_to_draw: Vec<Rc<Character>> = vec!();
        let mut string_to_draw = String::from("");
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
                    string_to_draw.push(c);
                    characters_to_draw.push(character.clone());
                    if characters_to_draw.len() == batch_draws_count {
                        self.draw_to_tex(&mut characters_to_draw, string_to_draw.clone(), font_size);
                        characters_to_draw.truncate(0);
                        string_to_draw = String::from("");
                    }
                    need_insert = true;
                    character
                }
            };
            if need_insert {
                self.char_tex_id_map.insert(key, character.clone());
            }
            character
        }).collect::<Vec<Rc<Character>>>().into_boxed_slice();
        if characters_to_draw.len() > 0 {
            self.draw_to_tex(&mut characters_to_draw, string_to_draw, font_size);
        }
        characters
    }
    pub fn free_chars(&mut self, _chars: &mut Box<[Rc<Character>]>) {
        // NOTE not freed immediately from hash map
        // do nothing here, just let rc auto drop
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
