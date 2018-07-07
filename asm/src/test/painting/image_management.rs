use std::rc::Rc;
use std::cell::RefCell;
use glayout::canvas::{Canvas, CanvasContext};
use glayout::canvas::element::{Element, EmptyElement, Image, Text};

lib_define_callback!(Step1 (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, _ret_code: i32) {
        let mut context = self.0.borrow_mut();
        let mut root_elem = context.get_root();
        let elem = {
            let cfg = context.get_canvas_config();
            let elem = element! {
                [&cfg] EmptyElement {
                    left = 10.;
                    top = 20.;
                    EmptyElement;
                    Image {
                        id = String::from("img");
                        width = 400.;
                        height = 400.;
                        .load("../resources/test.png");
                    };
                    Text {
                        font_size = 16.;
                        .set_text("Changing images");
                    };
                }
            };
            elem
        };
        root_elem.append(elem);
    }
});

lib_define_callback!(Step2 (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, _time: i32) {
        let mut context = self.0.borrow_mut();
        let image_node = context.get_node_by_id("img").unwrap();
        let mut image = image_node.elem().content_mut();
        let t = image.downcast_mut::<Image>().unwrap();
        t.load("../resources/lastleaf.png");
    }
});

pub fn init() {
    register_test_case!(module_path!(), {
        let mut canvas = Canvas::new(0);

        canvas.context(|ctx| {
            let pixel_ratio = ctx.get_device_pixel_ratio();
            ctx.set_canvas_size(800, 600, pixel_ratio);
            ctx.set_clear_color(0.5, 0.5, 0.5, 1.);
        });

        let rc_context = canvas.get_context();

        lib!(timeout(1000, lib_callback!(Step1(rc_context.clone()))));
        lib!(timeout(2000, lib_callback!(Step2(rc_context.clone()))));

        return 0;
    });
}
