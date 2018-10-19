use std::rc::Rc;
use std::cell::RefCell;
use glayout::canvas::{Canvas, CanvasContext};
use glayout::canvas::element::{Element, Empty, Image, Text};
use glayout::canvas::element::style::PositionType;

pub fn init() {
    register_test_case!(module_path!(), {
        let mut canvas = Canvas::new(0);

        canvas.ctx(|ctx| {
            let pixel_ratio = ctx.device_pixel_ratio();
            ctx.set_canvas_size(800, 600, pixel_ratio);
            ctx.set_clear_color(0.5, 0.5, 0.5, 1.);
        });

        let context = canvas.context();
        let mut context = context.borrow_mut();
        let mut root_elem = context.root();
        let elem = {
            let cfg = context.canvas_config();
            let elem = element! {
                [&cfg] Empty {
                    opacity: 0.7;
                    Empty;
                    Text {
                        set_text("The second image should cover the first image.");
                    };
                    Image {
                        position: PositionType::Absolute;
                        left: 100.;
                        top: 100.;
                        width: 400.;
                        height: 400.;
                        load("resources/test.png");
                    };
                    Image {
                        position: PositionType::Absolute;
                        left: 200.;
                        top: 200.;
                        width: 400.;
                        height: 400.;
                        load("resources/lastleaf.jpg");
                    };
                }
            };
            elem
        };
        root_elem.append(elem);

        return 0;
    });
}
