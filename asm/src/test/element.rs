use super::super::glayout::canvas::Canvas;
use super::super::glayout::element::{Element, EmptyElement, Image};
use super::super::glayout::tree::TreeNodeRc;
use super::super::glayout::frame::animation::{TimingAnimation, AnimationObject, LinearTiming};

pub fn test() -> i32 {
    let canvas = Canvas::new(1);
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
}
