use std::rc::Rc;
use std::cell::{RefCell, Cell};
use super::super::utils::PretendSend;
use super::character::CharacterManager;

lazy_static! {
    static ref IMAGE_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
}

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub tex_id_inc: i32,
    pub character_manager: PretendSend<Rc<RefCell<CharacterManager>>>,
}

impl CanvasConfig {
    pub fn new(index: i32, tex_size: i32, tex_count: i32, tex_max_draws: i32) -> Self {
        CanvasConfig {
            index,
            tex_size,
            tex_count,
            tex_max_draws,
            tex_id_inc: 0,
            character_manager: PretendSend::new(Rc::new(RefCell::new(CharacterManager::new()))),
        }
    }

    pub fn get_character_manager(&self) -> Rc<RefCell<CharacterManager>> {
        (*self.character_manager).clone()
    }

    pub fn alloc_tex_id(&mut self) -> i32 { // TODO impl create/release strategy
        let ret = self.tex_id_inc;
        self.tex_id_inc += 1;
        ret
    }
    pub fn alloc_image_id() -> i32 { // TODO impl create/release strategy
        let ret = IMAGE_ID_INC.get();
        IMAGE_ID_INC.set(ret + 1);
        ret
    }
}
