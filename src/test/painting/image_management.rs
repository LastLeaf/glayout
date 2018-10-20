use std::time;
use glayout;
use glayout::canvas::element::{Element, Empty, Image, Text};

pub fn init() {
    register_test_case!(module_path!(), rc_context, {
        let rc_context_1 = rc_context.clone();
        let rc_context_2 = rc_context.clone();

        let mut context = rc_context.borrow_mut();
        let pixel_ratio = context.device_pixel_ratio();
        context.set_canvas_size(800, 600, pixel_ratio);
        context.set_clear_color(0.5, 0.5, 0.5, 1.);

        glayout::set_timeout(move || {
            let mut context = rc_context_1.borrow_mut();
            let mut root_elem = context.root();
            let elem = {
                let cfg = context.canvas_config();
                let elem = element!(&cfg, Empty {
                    left: 10.;
                    top: 20.;
                    Empty;
                    Image {
                        id: String::from("img");
                        width: 400.;
                        height: 400.;
                        load("resources/test.png");
                    };
                    Text {
                        font_size: 16.;
                        set_text("Changing images");
                    };
                });
                elem
            };
            root_elem.append(elem);
        }, time::Duration::new(1, 0));

        glayout::set_timeout(move || {
            let mut context = rc_context_2.borrow_mut();
            let image_node = context.node_by_id("img").unwrap();
            let mut image = image_node.elem().content_mut();
            let t = image.downcast_mut::<Image>().unwrap();
            t.load("resources/lastleaf.jpg");
        }, time::Duration::new(2, 0));

        return 0;
    });
}
