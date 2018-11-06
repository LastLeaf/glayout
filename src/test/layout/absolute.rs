use std::time;
use glayout::canvas::element::{Element, Empty, Image, Text};
use glayout::canvas::element::style::{DisplayType, PositionType};

pub fn init() {
    register_test_case!(module_path!(), rc_context, {
        let mut context = rc_context.borrow_mut();

        let pixel_ratio = context.device_pixel_ratio();
        context.set_canvas_size(800, 600, pixel_ratio);
        context.set_clear_color(0.5, 1., 0.5, 1.);

        let elem = {
            let cfg = context.canvas_config();
            let elem = element! (&cfg, Empty {
                Text {
                    id: String::from("a");
                    position: PositionType::Absolute;
                    left: 10.;
                    top: 10.;
                    width: 50.;
                    set_text("Absolute Positioning");
                };
                color: (0., 0., 1., 0.5);
                Empty {
                    id: String::from("b");
                    display: DisplayType::Block;
                    position: PositionType::Absolute;
                    top: 100.;
                    left: 200.;
                    Text {
                        id: String::from("c");
                        font_size: 24.;
                        set_text("LARGE TEXT");
                    };
                    Image {
                        id: String::from("d");
                        width: 400.;
                        height: 400.;
                        load("resources/test.png");
                    };
                };
                Empty {
                    id: String::from("e");
                    position: PositionType::Absolute;
                    top: 100.;
                    left: 200.;
                    Text {
                        id: String::from("f");
                        font_size: 16.;
                        set_text("hahaha");
                    };
                };
            });
            elem
        };
        let mut root_elem = context.root();
        root_elem.append(elem);

        let init_time = time::Instant::now();
        let rc_context = rc_context.clone();
        frame!(move |time| {
            let mut context = rc_context.borrow_mut();
            let root = context.root();

            if context.touching() {
                match root.elem().node_under_point(context.touch_point()) {
                    Some(x) => {
                        println!("Touching: {:?}", x.elem().style().get_id());
                    },
                    None => {
                        println!("Touching nothing");
                    }
                }
            }

            let f = context.node_by_id("f").unwrap();
            let time = time.duration_since(init_time);
            let ts = time.as_secs() as f64 * 1000. + time.subsec_nanos() as f64 / 1_000_000.;
            f.elem().style_mut().transform_mut().reset().offset(ts / 1000. % 4. * 400., 0.);
            context.redraw();

            return true;
        });

        return 0;
    });
}
