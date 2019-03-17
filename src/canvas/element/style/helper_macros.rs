#![macro_use]

macro_rules! define_struct {
    ($($items:tt)*) => {
        pub struct ElementStyle {
            element: *mut Element,
            inline_class: Cell<ElementClass>,
            classes: Vec<Rc<ElementClass>>,
            tag_name: String,
            id: String,
            class: String,
            $($items)*
        }
    }
}

macro_rules! define_constructor {
    ($($items:tt)*) => {
        impl ElementStyle {
            pub(crate) fn new() -> Self {
                Self {
                    element: 0 as *mut Element,
                    inline_class: Cell::new(ElementClass::new()),
                    classes: vec![],
                    tag_name: String::new(),
                    id: String::new(),
                    class: String::new(),
                    $($items)*
                }
            }
        }
    }
}

macro_rules! impl_style_item {
    (
        $name:ident,
        $getter:ident,
        $setter:ident,
        $getter_advanced:ident,
        $setter_advanced:ident,
        $getter_inner:ident,
        $setter_inner:ident,
        $update_inherit:ident,
        $value_type:ty,
        $default_value_referrer:expr,
        $default_value:expr,
        $layout_dirty:expr,
        $inherit:expr
    ) => {
        pub(self) fn $getter_inner(&self) -> (StyleValueReferrer, $value_type) {
            if self.$name.is_dirty() {
                if self.$name.inherit() {
                    let value = {
                        let tree_node = self.node();
                        let parent = tree_node.parent();
                        match parent {
                            Some(p) => p.style.$name.get(),
                            None => ($default_value_referrer, $default_value),
                        }
                    };
                    self.$name.set(value.0, value.1);
                }
                self.$name.clear_dirty();
            }
            self.$name.get()
        }
        #[inline]
        pub fn $getter_advanced(&self) -> (StyleValueReferrer, $value_type) {
            self.$getter_inner()
        }
        #[inline]
        pub fn $getter(&self) -> $value_type {
            self.$getter_advanced().1
        }
        fn $update_inherit(tree_node: &mut ForestNode<Element>) {
            if $layout_dirty { tree_node.mark_layout_dirty() };
            let old_dirty = tree_node.style.$name.get_and_mark_dirty();
            if !old_dirty {
                tree_node.for_each_child_mut(|child| {
                    if child.style.$name.inherit() {
                        Self::$update_inherit(child);
                    }
                })
            }
        }
        pub(self) fn $setter_inner(&mut self, r: StyleValueReferrer, val: $value_type, inherit: bool) {
            let val = if r.is_absolute_or_relative() {
                val
            } else {
                $default_value
            };
            let changed = if inherit {
                let changed = !self.$name.inherit();
                self.$name.set_inherit(true);
                changed
            } else {
                let changed = r == self.$name.get_referrer() && val == *self.$name.get_value_ref();
                self.$name.set(r, val);
                changed
            };
            if changed {
                let tree_node = self.node_mut();
                Self::$update_inherit(tree_node);
            }
        }
        #[inline]
        pub fn $setter_advanced(&mut self, r: StyleValueReferrer, val: $value_type, inherit: bool) {
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val.clone()));
            self.$setter_inner(r, val, inherit);
        }
        #[inline]
        pub fn $setter(&mut self, val: $value_type) {
            self.$setter_advanced(StyleValueReferrer::Absolute, val, false);
        }
        #[inline]
        pub fn $name(&mut self, val: $value_type) {
            self.$setter(val);
        }
    }
}

macro_rules! impl_style_list {
    ($($items:tt)*) => {
        impl ElementStyle {
            $($items)*
        }
    }
}

macro_rules! impl_parent_updated_item {
    ($s:ident, $name:ident, $update_inherit:ident) => {
        if $s.$name.inherit() {
            Self::$update_inherit($s.node_mut());
        }
    }
}

macro_rules! impl_parent_updated {
    ($($items:tt)*) => {
        impl ElementStyle {
            $($items)*
        }
    }
}

macro_rules! define_style_name {
    ($($items:tt)*) => {
        #[allow(non_camel_case_types)]
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum StyleName {
            glayout_unrecognized = 0x00,
            $($items)*
        }
    }
}

macro_rules! impl_style_name {
    ($($items:tt)*) => {
        pub(self) fn apply_rule_from_class(style: &mut ElementStyle, name: &StyleName, value: &Box<dyn Any + Send>) {
            macro_rules! style_name {
                ($setter_inner: ident, $type: ty) => {
                    {
                        let (r, v) = value.downcast_ref::<StyleValue<$type>>().unwrap().get();
                        style.$setter_inner(r, v, false);
                    }
                }
            }
            match name {
                StyleName::glayout_unrecognized => { },
                $($items)*
            }
        }
    }
}
