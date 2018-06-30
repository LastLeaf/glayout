#![macro_use]

mod style;
pub type ElementStyle = style::ElementStyle;
mod bounding_rect;
pub type BoundingRect = bounding_rect::BoundingRect;

mod empty_element;
pub type EmptyElement = empty_element::EmptyElement;
mod image;
pub type Image = image::Image;
mod text;
pub type Text = text::Text;

use std::rc::Rc;
use std::fmt;
use downcast_rs::Downcast;
use super::CanvasConfig;

pub trait ElementContent: Downcast {
    fn name(&self) -> &'static str;
    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect);
}

impl_downcast!(ElementContent);

pub struct Element {
    pub style: ElementStyle,
    bounding_rect: BoundingRect,
    content: Box<ElementContent>,
}

impl Element {
    pub fn new(_cfg: &Rc<CanvasConfig>, content: Box<ElementContent>) -> Self {
        Element {
            style: ElementStyle::new(),
            bounding_rect: BoundingRect::new(),
            content,
        }
    }
    pub fn name(&self) -> &'static str {
        self.content.name()
    }
    pub fn draw(&mut self) {
        self.content.draw(&self.style, &self.bounding_rect);
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

#[macro_export]
macro_rules! __element_children {
    ($cfg:expr, $v:ident, $t:ident, ) => {};
    ($cfg:expr, $v:ident, $t:ident, $k:ident = $a:expr; $($r:tt)*) => {
        $v.get_mut().style.$k = $a;
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

#[macro_export]
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
