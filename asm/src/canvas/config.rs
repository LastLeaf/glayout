use std::rc::Rc;
use std::cell::RefCell;
use super::super::utils::PretendSend;
use super::character::CharacterManager;
use super::resource::ResourceManager;

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub device_pixel_ratio: f64,
    resource_manager: PretendSend<Rc<RefCell<ResourceManager>>>,
    character_manager: PretendSend<Rc<RefCell<CharacterManager>>>,
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
            resource_manager: PretendSend::new(resource_manager.clone()),
            character_manager: PretendSend::new(Rc::new(RefCell::new(CharacterManager::new(index, resource_manager)))),
        }
    }

    pub fn get_character_manager(&self) -> Rc<RefCell<CharacterManager>> {
        (*self.character_manager).clone()
    }

    pub fn get_resource_manager(&self) -> Rc<RefCell<ResourceManager>> {
        (*self.resource_manager).clone()
    }
}
