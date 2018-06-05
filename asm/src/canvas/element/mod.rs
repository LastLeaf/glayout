#![macro_use]

use downcast_rs::Downcast;

mod empty_element;
mod image;
pub type EmptyElement = empty_element::EmptyElement;
pub type Image = image::Image;

use std::fmt;
use super::CanvasConfig;

pub trait ElementContent: Downcast + Send + fmt::Debug {
    fn name(&self) -> &'static str;
    fn draw(&self, element: &Element);
}

impl_downcast!(ElementContent);

pub struct Element {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    content: Box<ElementContent>
}

impl Element {
    pub fn new(_cfg: &mut CanvasConfig, content: Box<ElementContent>) -> Self {
        Element {
            left: 0.,
            top: 0.,
            width: 0.,
            height: 0.,
            content
        }
    }
    pub fn name(&self) -> &'static str {
        self.content.name()
    }
    pub fn draw(&self) {
        self.content.draw(self);
        // TODO need impl
        // tree_node_rc.children.iter().for_each(|child| {
        //     child.get().draw();
        // });
    }
    pub fn get_content_ref<T: ElementContent>(&self) -> &T {
        self.content.downcast_ref::<T>().unwrap()
    }
    pub fn get_content_mut<T: ElementContent>(&mut self) -> &mut T {
        self.content.downcast_mut::<T>().unwrap()
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{name}", name = self.content.name())
    }
}

macro_rules! __element_children {
    ($cfg:expr, $v:ident, $t:ident, ) => {};
    ($cfg:expr, $v:ident, $t:ident, $k:ident = $a:expr; $($r:tt)*) => {
        $v.get_mut().$k = $a;
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, . $k:ident = $a:expr; $($r:tt)*) => {
        $v.get_mut().get_content_mut::<$t>().$k = $a;
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, . $k:ident ( $($a:expr),* ); $($r:tt)*) => {
        $v.get_mut().get_content_mut::<$t>().$k($($a),*);
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident; $($r:tt)*) => {
        __element_children! ($cfg, $v, $t, $e {}; $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident { $($c:tt)* }; $($r:tt)*) => {
        let mut temp_element_child = __element_tree! ( $cfg, $e { $($c)* });
        $v.append(temp_element_child);
        __element_children! ($cfg, $v, $t, $($r)*);
    }
}

macro_rules! __element_tree {
    ($cfg:expr, $e:ident) => {
        __element_tree! ($cfg, $e {})
    };
    ($cfg:expr, $e:ident { $($c:tt)* }) => {{
        let mut temp_content = Box::new($e::new($cfg));
        let mut temp_element = TreeNodeRc::new(Element::new($cfg, temp_content));
        {
            let mut _temp_element_inner = temp_element.clone();
            __element_children! ($cfg, _temp_element_inner, $e, $($c)*);
        }
        temp_element
    }}
}

#[macro_export]
macro_rules! element {
    ([$cfg:expr] $($c:tt)*) => {{
        __element_tree! ($cfg, $($c)*)
    }}
}

pub mod test {
    use super::{Element, EmptyElement, Image};
    use super::super::Canvas;
    use super::super::super::tree::{TreeNodeRc};

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
}
