use std::cell::Cell;
use super::super::utils::PretendSend;

lazy_static! {
    static ref IMAGE_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
}

pub struct ResourceManager {
    tex_id_inc: i32,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            tex_id_inc: 0
        }
    }

    pub fn alloc_tex_id(&mut self) -> i32 { // TODO impl create/release strategy
        let ret = self.tex_id_inc;
        self.tex_id_inc += 1;
        ret
    }
    pub fn alloc_image_id(&mut self) -> i32 { // TODO impl create/release strategy
        let ret = IMAGE_ID_INC.get();
        IMAGE_ID_INC.set(ret + 1);
        ret
    }
}
