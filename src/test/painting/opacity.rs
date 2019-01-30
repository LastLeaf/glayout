use glayout::canvas::element::{Element, Empty, Image, Text};
use glayout::canvas::element::style::{PositionType, DisplayType};

pub fn init() {
    register_test_case!(module_path!(), rc_context, {
        let mut context = rc_context.borrow_mut();

        let pixel_ratio = context.device_pixel_ratio();
        context.set_canvas_size(800, 600, pixel_ratio);
        context.set_clear_color(0.5, 0.5, 0.5, 1.);

        let root_elem = context.root();
        let elem = {
            let cfg = context.canvas_config();
            let mut root = context.root().borrow_mut();
            let elem = element!(&mut root, &cfg, Empty {
                Empty;
                Text {
                    set_text("The second image should cover the first image.");
                };
                Empty {
                    position: PositionType::Absolute;
                    display: DisplayType::Block;
                    opacity: 0.7;
                    left: -10.;
                    top: -10.;
                    Image {
                        position: PositionType::Absolute;
                        left: 100.;
                        top: 10.;
                        width: 400.;
                        height: 400.;
                        load("resources/test.png");
                    };
                    Image {
                        position: PositionType::Absolute;
                        display: DisplayType::Block;
                        left: 190.;
                        top: 190.;
                        width: 400.;
                        height: 400.;
                        load("resources/lastleaf.jpg");
                    };
                    Text {
                        position: PositionType::Absolute;
                        display: DisplayType::Block;
                        background_color: (0., 0.5, 0., 0.2);
                        opacity: 0.8;
                        left: 140.;
                        top: 180.;
                        height: 100.;
                        color: (1., 0., 0., 1.);
                        font_size: 32.;
                        set_text("center");
                    };
                };
            });
            elem
        };
        root_elem.borrow_mut().append(elem);

        return 0;
    });
}
