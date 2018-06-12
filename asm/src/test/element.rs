use glayout::canvas::Canvas;
use glayout::canvas::element::{Element, EmptyElement, Image, Text};
use glayout::tree::TreeNodeRc;

pub fn init() {
    register_test_case!(module_path!(), {
        let mut canvas = Canvas::new(0);

        canvas.context(|ctx| {
            ctx.set_canvas_size(800, 800);
            ctx.set_clear_color(0.5, 0.5, 0.5, 1.);
        });

        let arc_context = canvas.get_context();
        let mut context = arc_context.lock().unwrap();
        let elem = {
            let cfg = context.get_canvas_config();
            let elem = element! {
                 [cfg] EmptyElement {
                    left = 10.;
                    top = 20.;
                    EmptyElement;
                    Image {
                        width = 400.;
                        height = 400.;
                        .load("../resources/lastleaf.png");
                    };
                    EmptyElement {
                        Text {
                            font_size = 32;
                            .set_text("Hello Lena~");
                        };
                        top = 750.;
                    };
                }
            };
            elem
        };
        let mut root_elem = context.get_root();
        root_elem.append(elem);
        return 0;
    });
}
