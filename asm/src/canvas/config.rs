pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub image_id_inc: i32
}

impl CanvasConfig {
    pub fn alloc_image_id(&mut self) -> i32 {
        let ret = self.image_id_inc + 1;
        self.image_id_inc += 1;
        ret
    }
}
