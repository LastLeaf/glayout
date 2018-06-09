use super::super::utils::PretendSend;
use std::cell::Cell;

lazy_static! {
    static ref IMAGE_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
    static ref FONT_FAMILY_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
}

#[derive(Default)]
pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub tex_id_inc: i32,
}

impl CanvasConfig {
    pub fn new(index: i32, tex_size: i32, tex_count: i32, tex_max_draws: i32) -> Self {
        CanvasConfig {
            index,
            tex_size,
            tex_count,
            tex_max_draws,
            ..Default::default()
        }
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
    pub fn alloc_font_family_id() -> i32 {
        let ret = FONT_FAMILY_ID_INC.get();
        FONT_FAMILY_ID_INC.set(ret + 1);
        ret
    }
}
