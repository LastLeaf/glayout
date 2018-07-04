use std::cell::RefCell;
use std::collections::{HashMap, BinaryHeap};
use super::super::utils::PretendSend;

lazy_static! {
    static ref IMAGE_ID_INC: PretendSend<RefCell<ResourceIdAllocator>> = PretendSend::new(RefCell::new(ResourceIdAllocator::new()));
}

const TEX_SHADER_INDEX_MAX: i32 = 16;

struct ResourceIdAllocator {
    inc: i32,
    released: BinaryHeap<i32>,
}

impl ResourceIdAllocator {
    fn new() -> Self {
        Self {
            inc: 0,
            released: BinaryHeap::new(),
        }
    }
    fn alloc(&mut self) -> i32 {
        match self.released.pop() {
            None => {
                let ret = self.inc;
                self.inc += 1;
                ret
            },
            Some(x) => {
                -x
            }
        }
    }
    fn free(&mut self, id: i32) {
        self.released.push(-id);
    }
}

pub struct ResourceManager {
    canvas_index: i32,
    tex_max_draws: i32,
    tex_id_allocator: ResourceIdAllocator,
    pending_draws: i32,
    used_shader_tex: i32,
    tex_shader_index_map: HashMap<i32, i32>,
}

impl ResourceManager {
    pub fn new(canvas_index: i32, tex_max_draws: i32) -> Self {
        Self {
            canvas_index,
            tex_max_draws,
            tex_id_allocator: ResourceIdAllocator::new(),
            pending_draws: 0,
            used_shader_tex: 0,
            tex_shader_index_map: HashMap::new(),
        }
    }

    pub fn alloc_tex_id(&mut self) -> i32 {
        let ret = self.tex_id_allocator.alloc();
        debug!("Alloc tex id: {}", ret);
        ret
    }
    pub fn free_tex_id(&mut self, tex_id: i32) {
        self.tex_id_allocator.free(tex_id);
        debug!("Free tex id: {}", tex_id);
    }

    pub fn alloc_image_id() -> i32 {
        let ret = IMAGE_ID_INC.borrow_mut().alloc();
        debug!("Alloc image id: {}", ret);
        ret
    }
    pub fn free_image_id(image_id: i32) {
        IMAGE_ID_INC.borrow_mut().free(image_id);
        debug!("Free image id: {}", image_id);
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
