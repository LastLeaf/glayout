use std::cell::RefCell;
use std::collections::{BTreeMap, BinaryHeap};
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DrawState {
    pub color: (f32, f32, f32, f32),
    pub alpha: f32,
}

impl DrawState {
    #[inline]
    pub fn new() -> Self {
        Self {
            color: (-1., -1., -1., -1.),
            alpha: -1.,
        }
    }
    #[inline]
    pub fn color(&mut self, color: (f32, f32, f32, f32)) -> &mut Self {
        self.color = color;
        self
    }
    #[inline]
    pub fn alpha(&mut self, alpha: f32) -> &mut Self {
        self.alpha = alpha;
        self
    }
    #[inline]
    pub fn get_alpha(&mut self) -> f32 {
        self.alpha
    }
    #[inline]
    pub fn mul_alpha(&mut self, alpha: f32) -> &mut Self {
        if self.alpha >= 0. {
            self.alpha *= alpha;
        }
        self
    }
}

pub struct ResourceManager {
    canvas_index: i32,
    tex_max_draws: i32,
    tex_id_allocator: ResourceIdAllocator,
    pending_draws: i32,
    used_shader_tex: i32,
    tex_shader_index_map: BTreeMap<i32, i32>,
    current_draw_state: DrawState,
}

impl ResourceManager {
    pub fn new(canvas_index: i32, tex_max_draws: i32) -> Self {
        Self {
            canvas_index,
            tex_max_draws,
            tex_id_allocator: ResourceIdAllocator::new(),
            pending_draws: 0,
            used_shader_tex: 0,
            tex_shader_index_map: BTreeMap::new(),
            current_draw_state: *DrawState::new().color((0., 0., 0., 1.)).alpha(1.),
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
    pub fn draw_state(&self) -> DrawState {
        self.current_draw_state
    }
    pub fn set_draw_state(&mut self, ds: &mut DrawState) {
        if ds.color.0 < 0. { ds.color = self.current_draw_state.color; }
        if ds.alpha < 0. { ds.alpha = self.current_draw_state.alpha; }
        if self.current_draw_state == *ds {
            return;
        }
        self.current_draw_state = *ds;
        self.flush_draw();
        lib!(tex_set_draw_state(self.canvas_index, ds.color.0 * ds.color.3, ds.color.1 * ds.color.3, ds.color.2 * ds.color.3, ds.color.3, ds.alpha));
    }
    #[inline]
    pub fn request_draw(&mut self,
        tex_id: i32, use_color: bool,
        tex_left: f64, tex_top: f64, tex_width: f64, tex_height: f64,
        (left, top, width, height): (f64, f64, f64, f64)
    ) {
        // TODO ignore draws that exceed viewport
        if self.pending_draws == self.tex_max_draws {
            self.flush_draw();
        }
        let tex_shader_index;
        if tex_id >= 0 {
            let tex_shader_index_option = self.tex_shader_index_map.get(&tex_id).map(|i| {*i});
            tex_shader_index = match tex_shader_index_option {
                None => {
                    if self.used_shader_tex == TEX_SHADER_INDEX_MAX {
                        self.flush_draw();
                    }
                    let t = self.used_shader_tex;
                    self.tex_shader_index_map.insert(tex_id, t);
                    self.used_shader_tex += 1;
                    t
                },
                Some(ref t) => {
                    *t
                }
            };
        } else {
            tex_shader_index = tex_id;
        }
        lib!(tex_draw(self.canvas_index,
            self.pending_draws, tex_shader_index + (if tex_id < 0 || use_color { 0 } else { 256 }),
            tex_left as f32, tex_top as f32, tex_width as f32, tex_height as f32,
            left as f32, top as f32, width as f32, height as f32
        ));
        self.pending_draws += 1;
    }
    #[inline]
    pub fn flush_draw(&mut self) {
        if self.pending_draws == 0 { return }
        let mut t_max = 0;
        for (tex_id, t) in self.tex_shader_index_map.iter() {
            lib!(tex_set_active_texture(self.canvas_index, *t, *tex_id));
            if t_max < *t {
                t_max = *t;
            }
        }
        for t in t_max + 1 .. TEX_SHADER_INDEX_MAX {
            lib!(tex_set_active_texture(self.canvas_index, t, -1));
        }
        lib!(tex_draw_end(self.canvas_index, self.pending_draws));
        self.pending_draws = 0;
        self.used_shader_tex = 0;
        self.tex_shader_index_map.clear();
    }
    #[inline]
    pub fn bind_rendering_target(&mut self, tex_id: i32, width: i32, height: i32) {
        self.flush_draw();
        lib!(tex_bind_rendering_target(self.canvas_index, tex_id, width, height));
    }
    #[inline]
    pub fn unbind_rendering_target(&mut self) {
        self.flush_draw();
        lib!(tex_unbind_rendering_target(self.canvas_index));
    }
}
