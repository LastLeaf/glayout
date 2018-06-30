use std::rc::Rc;
use std::cell::{Cell, RefCell};
use super::character::CharacterManager;
use super::resource::ResourceManager;

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub device_pixel_ratio: f64,
    clear_color: Cell<(f32, f32, f32, f32)>,
    dirty: Cell<bool>,
    pending_draws: Cell<i32>,
    resource_manager: Rc<RefCell<ResourceManager>>,
    character_manager: Rc<RefCell<CharacterManager>>,
}

impl CanvasConfig {
    pub fn new(index: i32, tex_size: i32, tex_count: i32, tex_max_draws: i32, device_pixel_ratio: f64) -> Self {
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new()));
        CanvasConfig {
            index,
            tex_size,
            tex_count,
            tex_max_draws,
            device_pixel_ratio,
            clear_color: Cell::new((1., 1., 1., 0.)),
            dirty: Cell::new(false),
            pending_draws: Cell::new(0),
            resource_manager: resource_manager.clone(),
            character_manager: Rc::new(RefCell::new(CharacterManager::new(index, resource_manager))),
        }
    }

    #[inline]
    pub fn mark_dirty(&self) {
        self.dirty.set(true);
    }
    #[inline]
    pub fn clear_dirty(&self) -> bool {
        let ret = self.dirty.get();
        self.dirty.set(false);
        ret
    }

    #[inline]
    pub fn set_clear_color(&self, color: (f32, f32, f32, f32)) {
        self.clear_color.set(color);
    }
    #[inline]
    pub fn get_clear_color(&self) -> (f32, f32, f32, f32) {
        self.clear_color.get()
    }

    #[inline]
    pub fn request_draw(&self, tex_id: i32, tex_left: f64, tex_top: f64, tex_width: f64, tex_height: f64, left: f64, top: f64, width: f64, height: f64) {
        let mut draw_count = self.pending_draws.get();
        if draw_count == 16 {
            self.flush_draw();
            draw_count = 0;
        }
        lib!(tex_draw(self.index, draw_count, tex_id,
            tex_left, tex_top, tex_width, tex_height,
            left, top, width, height
        ));
        self.pending_draws.set(draw_count + 1);
    }
    #[inline]
    pub fn flush_draw(&self) {
        let draw_count = self.pending_draws.get();
        if draw_count > 0 {
            lib!(tex_draw_end(self.index, draw_count));
        }
        self.pending_draws.set(0);
    }

    #[inline]
    pub fn get_character_manager(&self) -> Rc<RefCell<CharacterManager>> {
        self.character_manager.clone()
    }
    #[inline]
    pub fn get_resource_manager(&self) -> Rc<RefCell<ResourceManager>> {
        self.resource_manager.clone()
    }
}
