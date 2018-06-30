use std::cmp;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::str::Chars;
use std::ffi::CString;
use super::super::utils::PretendSend;
use super::resource::ResourceManager;
use std::collections::HashMap;

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
    fn normalize_size(&self, total_width: f64, total_height: f64) {
        self.left.set(self.left.get() / total_width);
        self.top.set(self.top.get() / total_height);
        self.width.set(self.width.get() / total_width);
        self.height.set(self.height.get() / total_height);
    }
    #[inline]
    pub fn get_position(&self) -> (f64, f64, f64, f64, f64, f64) {
        (self.left.get(), self.top.get(), self.width.get(), self.height.get(), self.natural_width.get(), self.natural_height.get())
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

#[inline]
fn get_default_line_height(font_size: i32) -> f64 {
    (font_size as f64 * 1.5).ceil()
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

    fn draw_to_tex(&self, characters: &mut Vec<Rc<Character>>, whole_string: String, font_size: i32, count_per_row: usize) {
        let mut left: f64 = 0.;
        let mut top: f64 = 0.;
        let mut total_width: f64 = 0.;
        let mut cur_col = 0;
        let tex_id = self.resource_manager.borrow_mut().alloc_tex_id();
        let line_height = get_default_line_height(font_size);
        characters.iter().for_each(|character| {
            let mut s = String::new();
            s.push(character.unicode);
            let width = lib!(text_get_width(CString::new(s).unwrap().into_raw())); // FIXME should be able to batch
            character.set_position(left, top, width, line_height, width, line_height);
            character.alloc_tex(tex_id);
            left += width;
            cur_col += 1;
            if cur_col == count_per_row {
                total_width = if total_width > left { total_width } else { left };
                left = 0.;
                top += line_height;
                cur_col = 0;
            }
        });
        total_width = total_width.ceil();
        let total_height = if cur_col > 0 { top + line_height } else { top };
        lib!(text_to_tex(self.canvas_index, tex_id, CString::new(whole_string).unwrap().into_raw(), total_width as i32, total_height as i32, line_height as i32));
        characters.iter().for_each(|character| {
            character.normalize_size(total_width, total_height);
        });
    }

    pub fn alloc_chars(&mut self, font_family_id: i32, font_size: i32, font_style: FontStyle, chars: Chars) -> Box<[Rc<Character>]> {
        let font_size = cmp::max(font_size, MIN_FONT_SIZE);
        let line_height = get_default_line_height(font_size);
        lib!(text_set_font(font_size, line_height as i32, font_family_id, (font_style == FontStyle::Italic || font_style == FontStyle::BoldItalic) as i32, (font_style == FontStyle::Bold || font_style == FontStyle::BoldItalic) as i32));
        let batch_char_count_per_row: usize = (BG_CANVAS_SIZE / (font_size * 2)) as usize;
        let batch_char_rows: usize = (BG_CANVAS_SIZE as f64 / line_height) as usize;
        let mut batch_count_in_row: usize = 0;
        let mut batch_cur_row: usize = 0;
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
                    if c < ' ' {
                        character.set_position(0., 0., 0., 0., 0., line_height);
                    } else {
                        batch_count_in_row += 1;
                        string_to_draw.push(c);
                        characters_to_draw.push(character.clone());
                        if batch_count_in_row == batch_char_count_per_row {
                            batch_count_in_row = 0;
                            batch_cur_row += 1;
                            string_to_draw.push('\n');
                            if batch_cur_row == batch_char_rows {
                                self.draw_to_tex(&mut characters_to_draw, string_to_draw.clone(), font_size, batch_char_count_per_row);
                                batch_cur_row = 0;
                                characters_to_draw.truncate(0);
                                string_to_draw = String::from("");
                            }
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
            self.draw_to_tex(&mut characters_to_draw, string_to_draw, font_size, batch_char_count_per_row);
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
