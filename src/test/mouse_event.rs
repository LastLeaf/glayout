use glayout::frame;

pub fn init() {
    register_test_case!(module_path!(), rc_ctx, {
        let mut ctx = rc_ctx.borrow_mut();
        let pixel_ratio = ctx.device_pixel_ratio();
        ctx.set_canvas_size(800, 600, pixel_ratio);
        ctx.set_clear_color(0.5, 1., 0.5, 1.);

        let rc_ctx = rc_ctx.clone();
        frame!(move |_time| {
            let ctx = rc_ctx.borrow_mut();
            if ctx.touching() {
                println!("Touching: {:?}", ctx.touch_point());
            }
            return true;
        });

        return 0;
    });
}
