use std::cell::Cell;
use std::collections::HashMap;
use super::super::utils::PretendSend;

lazy_static! {
    static ref IMAGE_ID_INC: PretendSend<Cell<i32>> = PretendSend::new(Cell::new(0));
}

const TEX_SHADER_INDEX_MAX: i32 = 16;

pub struct ResourceManager {
    canvas_index: i32,
    tex_max_draws: i32,
    tex_id_inc: i32,
    pending_draws: i32,
    used_shader_tex: i32,
    tex_shader_index_map: HashMap<i32, i32>,
}

impl ResourceManager {
    pub fn new(canvas_index: i32, tex_max_draws: i32) -> Self {
        Self {
            canvas_index,
            tex_max_draws,
            tex_id_inc: 0,
            pending_draws: 0,
            used_shader_tex: 0,
            tex_shader_index_map: HashMap::new(),
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

    #[inline]
    pub fn request_draw(&mut self, tex_id: i32, tex_left: f64, tex_top: f64, tex_width: f64, tex_height: f64, left: f64, top: f64, width: f64, height: f64) {
        // TODO ignore draws that exceed viewport
        if self.pending_draws == self.tex_max_draws {
            self.flush_draw();
        }
        let tex_shader_index_option = self.tex_shader_index_map.get(&tex_id).map(|i| {*i});
        let tex_shader_index: i32 = match tex_shader_index_option {
            None => {
                if self.used_shader_tex == TEX_SHADER_INDEX_MAX {
                    self.flush_draw();
                }
                let t = self.used_shader_tex;
                self.tex_shader_index_map.insert(tex_id, t);
                lib!(tex_set_active_texture(self.canvas_index, t, tex_id));
                self.used_shader_tex += 1;
                t
            },
            Some(ref t) => {
                *t
            }
        };
        lib!(tex_draw(self.canvas_index, self.pending_draws, tex_shader_index,
            tex_left, tex_top, tex_width, tex_height,
            left, top, width, height
        ));
        self.pending_draws += 1;
    }
    #[inline]
    pub fn flush_draw(&mut self) {
        if self.pending_draws > 0 {
            lib!(tex_draw_end(self.canvas_index, self.pending_draws));
        }
        self.pending_draws = 0;
        self.used_shader_tex = 0;
        self.tex_shader_index_map.clear();
    }
}
