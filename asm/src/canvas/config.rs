use std::rc::Rc;
use std::cell::{Cell, RefCell};
use super::super::get_window_size;
use super::character::CharacterManager;
use super::resource::ResourceManager;

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub device_pixel_ratio: f64,
    pub window_size: Cell<(f64, f64)>,
    clear_color: Cell<(f32, f32, f32, f32)>,
    resource_manager: Rc<RefCell<ResourceManager>>,
    character_manager: Rc<RefCell<CharacterManager>>,
}

impl CanvasConfig {
    pub fn new(index: i32, tex_size: i32, tex_count: i32, tex_max_draws: i32, device_pixel_ratio: f64) -> Self {
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(index, tex_max_draws)));
        CanvasConfig {
            index,
            tex_size,
            tex_count,
            tex_max_draws,
            device_pixel_ratio,
            window_size: Cell::new(get_window_size()),
            clear_color: Cell::new((1., 1., 1., 0.)),
            resource_manager: resource_manager.clone(),
            character_manager: Rc::new(RefCell::new(CharacterManager::new(index, resource_manager))),
        }
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
    pub fn get_character_manager(&self) -> Rc<RefCell<CharacterManager>> {
        self.character_manager.clone()
    }
    #[inline]
    pub fn get_resource_manager(&self) -> Rc<RefCell<ResourceManager>> {
        self.resource_manager.clone()
    }
}
