use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::str::Chars;
use super::super::utils::PretendSend;
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
    tex_id: i32
}

impl Character {
    pub fn new(unicode: char, font_family_id: i32, font_size: i32, font_style: FontStyle) -> Self {
        // TODO generate tex
        Self {
            unicode,
            font_family_id,
            font_size,
            font_style,
            tex_id: -1,
        }
    }

    fn release() {
        unimplemented!();
    }

    #[inline]
    pub fn get_tex_id(&self) -> i32 {
        self.tex_id
    }
}

pub struct CharacterManager {
    char_tex_id_map: HashMap<CharacterKey, Rc<Character>>,
}

impl CharacterManager {
    pub fn new() -> Self {
        Self {
            char_tex_id_map: HashMap::new(),
        }
    }

    pub fn alloc_chars(&mut self, font_family_id: i32, font_size: i32, font_style: FontStyle, chars: Chars) -> Vec<Rc<Character>> {
        chars.map(|c: char| {
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
                    Rc::new(Character::new(c, font_family_id, font_size, font_style))
                }
            }
        }).collect()
    }
    pub fn free_chars(&mut self, chars: &mut Vec<Rc<Character>>) {
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
