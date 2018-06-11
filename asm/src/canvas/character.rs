use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::str::Chars;
use std::ffi::CString;
use super::super::utils::PretendSend;
use super::resource::ResourceManager;
use std::collections::HashMap;

lazy_static! {
    static ref FONT_FAMILY_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
    static ref FONT_FAMILY_MAP: PretendSend<RefCell<HashMap<String, i32>>> = PretendSend::new(RefCell::new(HashMap::new()));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Bold,
    Italics,
    BoldItalics,
}

#[derive(PartialEq, Eq, Hash)]
struct CharacterKey {
    pub unicode: char,
    pub font_family_id: i32,
    pub font_size: i32,
    pub font_style: FontStyle,
}

pub struct Character {
    unicode: char,
    font_family_id: i32,
    font_size: i32,
    font_style: FontStyle,
    width: i32,
    tex_id: i32
}

impl Character {
    pub fn new(unicode: char, font_family_id: i32, font_size: i32, font_style: FontStyle) -> Self {
        Self {
            unicode,
            font_family_id,
            font_size,
            font_style,
            width: 0,
            tex_id: -1,
        }
    }

    fn set_width(&mut self, width: i32) {
        self.width = width;
    }
    fn alloc_tex(&mut self, tex_id: i32) {

    }
    fn free_tex(&mut self) {
        unimplemented!();
    }

    #[inline]
    pub fn get_tex_id(&self) -> i32 {
        self.tex_id
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

    pub fn alloc_chars(&mut self, font_family_id: i32, font_size: i32, font_style: FontStyle, chars: Chars) -> Box<[Rc<Character>]> {
        let mut need_drawing_string = String::from("");
        let mut need_drawing_pos: Vec<usize> = vec!();
        let mut left = 0;
        let mut index = 1;
        let characters = chars.map(|c| {
            let mut key = CharacterKey {
                unicode: c,
                font_family_id,
                font_size,
                font_style,
            };
            match self.char_tex_id_map.get(&mut key) {
                Some(x) => {
                    x.clone()
                },
                None => {
                    let character = Rc::new(Character::new(c, font_family_id, font_size, font_style));
                    need_drawing_string.push(c);
                    need_drawing_pos.push(index);
                    index += 1;
                    character
                }
            }
        }).collect::<Vec<Rc<Character>>>().into_boxed_slice();
        let total_width = left;
        lib!(text_draw_in_canvas(CString::new(need_drawing_string).unwrap().into_raw(), total_width, font_size)); // FIXME check whether can be draw in one time
        let mut left = 0;
        let mut index = 0;
        need_drawing_string.chars().for_each(|c| {
            let character = characters[need_drawing_pos[index]];
            let s = String::new();
            s.push(c);
            let width = lib!(text_get_width(CString::new(s).unwrap().into_raw())); // FIXME should be able to batch
            character.set_width(width);
            let tex_id = self.resource_manager.borrow_mut().alloc_tex_id();
            character.alloc_tex(tex_id);
            lib!(tex_from_text(self.canvas_index, tex_id, left, 0, width, font_size));
            left += width;
            index += 1;
        });
        characters
    }
    pub fn free_chars(&mut self, chars: &mut Box<[Rc<Character>]>) {
        // NOTE not freed immediately from hash map
        unimplemented!();
    }

    fn alloc_font_family_id() -> i32 {
        let ret = FONT_FAMILY_ID_INC.get();
        FONT_FAMILY_ID_INC.set(ret + 1);
        ret
    }
    pub fn get_font_family_id(name: String) -> i32 {
        // NOTE font-family is never released
        unimplemented!();
    }
}
