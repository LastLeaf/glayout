use glayout::canvas::Canvas;
use glayout::canvas::element::{Element, EmptyElement, Image};
use glayout::tree::TreeNodeRc;

pub fn init() {
    register_test_case!(module_path!(), {
        let canvas = Canvas::new(0);
        let arc_context = canvas.get_context();
        let mut context = arc_context.lock().unwrap();
        let elem = {
            let cfg = context.get_canvas_config();
            let elem = element! {
                 [cfg] EmptyElement {
                    left = 10.;
                    top = 20.;
                    EmptyElement;
                    EmptyElement {
                        EmptyElement;
                        top = 20.;
                    };
                    Image {
                        width = 100.;
                        height = 100.;
                        .load("../resources/lastleaf.png");
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
