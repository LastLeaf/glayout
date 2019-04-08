#![macro_use]

macro_rules! define_struct {
    ($($items:tt)*) => {
        pub struct ElementStyle {
            element: *mut Element,
            inline_class: Cell<ElementClass>,
            classes: RefCell<Vec<Rc<ElementClass>>>,
            tag_name: String,
            id: String,
            class: String,
            all_dirty: Cell<bool>, // all inherit are marked dirty
            class_dirty: Cell<ClassDirtyStatus>, // classes should be re-evaluated
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
                    classes: RefCell::new(vec![]),
                    tag_name: String::new(),
                    id: String::new(),
                    class: String::new(),
                    all_dirty: Cell::new(true),
                    class_dirty: Cell::new(ClassDirtyStatus::NotDirty),
                    $($items)*
                }
            }
        }
    }
}

// NOTE get_xxx might be outdated if layout dirty
macro_rules! impl_style_item_getter {
    ($getter:ident, $getter_advanced:ident, $value_type:ty, none) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            self.$getter_advanced().1
        }
    };
    ($getter:ident, $getter_advanced:ident, $value_type:ty, horizontal) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            let element = self.element();
            let (r, v) = self.$getter_advanced();
            match r {
                StyleValueReferrer::RelativeToParentSize => element.get_base_width() as $value_type * v,
                StyleValueReferrer::RelativeToParentFontSize => element.get_base_font_size() as $value_type * v,
                StyleValueReferrer::RelativeToViewportWidth => element.canvas_config.canvas_size.get().width() as $value_type * v,
                StyleValueReferrer::RelativeToViewportHeight => element.canvas_config.canvas_size.get().height() as $value_type * v,
                _ => v,
            }
        }
    };
    ($getter:ident, $getter_advanced:ident, $value_type:ty, vertical) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            let element = self.element();
            let (r, v) = self.$getter_advanced();
            match r {
                StyleValueReferrer::RelativeToParentSize => element.get_base_height() as $value_type * v,
                StyleValueReferrer::RelativeToParentFontSize => element.get_base_font_size() as $value_type * v,
                StyleValueReferrer::RelativeToViewportWidth => element.canvas_config.canvas_size.get().width() as $value_type * v,
                StyleValueReferrer::RelativeToViewportHeight => element.canvas_config.canvas_size.get().height() as $value_type * v,
                _ => v,
            }
        }
    };
    ($getter:ident, $getter_advanced:ident, $value_type:ty, flex_direction) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            let element = self.element();
            let (r, v) = self.$getter_advanced();
            match r {
                StyleValueReferrer::RelativeToParentSize => {
                    let s = match element.style.get_flex_direction() {
                        FlexDirectionType::Column | FlexDirectionType::ColumnReverse => {
                            element.get_base_height()
                        },
                        _ => {
                            element.get_base_width()
                        }
                    };
                    s as $value_type * v
                },
                StyleValueReferrer::RelativeToParentFontSize => element.get_base_font_size() as $value_type * v,
                StyleValueReferrer::RelativeToViewportWidth => element.canvas_config.canvas_size.get().width() as $value_type * v,
                StyleValueReferrer::RelativeToViewportHeight => element.canvas_config.canvas_size.get().height() as $value_type * v,
                _ => v,
            }
        }
    };
    ($getter:ident, $getter_advanced:ident, $value_type:ty, font_size) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            let element = self.element();
            let (r, v) = self.$getter_advanced();
            match r {
                StyleValueReferrer::RelativeToParentSize | StyleValueReferrer::RelativeToParentFontSize => element.get_base_font_size() as $value_type * v,
                StyleValueReferrer::RelativeToViewportWidth => element.canvas_config.canvas_size.get().width() as $value_type * v,
                StyleValueReferrer::RelativeToViewportHeight => element.canvas_config.canvas_size.get().height() as $value_type * v,
                _ => v,
            }
        }
    };
    ($getter:ident, $getter_advanced:ident, $value_type:ty, parent_font_size) => {
        #[inline]
        pub fn $getter(&self) -> $value_type {
            let element = self.element();
            let (r, v) = self.$getter_advanced();
            match r {
                StyleValueReferrer::RelativeToParentSize | StyleValueReferrer::RelativeToParentFontSize => {
                    match element.node().parent() {
                        Some(parent) => {
                            parent.get_base_font_size() as $value_type * v
                        },
                        None => {
                            DEFAULT_FONT_SIZE as $value_type * v
                        }
                    }
                },
                StyleValueReferrer::RelativeToViewportWidth => element.canvas_config.canvas_size.get().width() as $value_type * v,
                StyleValueReferrer::RelativeToViewportHeight => element.canvas_config.canvas_size.get().height() as $value_type * v,
                _ => v,
            }
        }
    };
}

macro_rules! impl_style_item {
    (
        $name:ident,
        $setter:ident,
        $getter_advanced:ident,
        $setter_advanced:ident,
        $setter_set_inherit: ident,
        $getter_inner:ident,
        $setter_update:ident,
        $update_inherit:ident,
        $value_type:ty,
        $default_value_referrer:expr,
        $default_value:expr,
        $layout_dirty:expr,
        $inherit:expr,
        $font_size_inherit:expr,
    ) => {
        // getters
        pub(self) fn $getter_inner(&self) -> (StyleValueReferrer, $value_type) {
            self.check_and_update_classes();
            if self.$name.is_dirty() {
                if self.$name.inherit() {
                    let value = {
                        let tree_node = self.node();
                        let parent = tree_node.parent();
                        match parent {
                            Some(p) => p.style.$getter_inner(),
                            None => ($default_value_referrer, $default_value),
                        }
                    };
                    self.$name.set(value.0, value.1);
                }
                self.all_dirty.set(false);
                self.$name.clear_dirty();
            }
            self.$name.get()
        }
        #[inline]
        pub fn $getter_advanced(&self) -> (StyleValueReferrer, $value_type) {
            self.$getter_inner()
        }
        // mark child dirty if it inherit or relative to the style
        fn $update_inherit(tree_node: &mut ForestNode<Element>) {
            if $layout_dirty {
                tree_node.mark_layout_dirty();
            }
            let old_dirty = tree_node.style.$name.get_and_mark_dirty();
            if !old_dirty {
                tree_node.for_each_child_mut(|child| {
                    // changing font_size causes all nodes dirty if font_size related
                    if child.style.$name.inherit() || ($font_size_inherit && child.style.$name.get_referrer().is_parent_relative()) {
                        Self::$update_inherit(child);
                    }
                })
            }
        }
        // setters
        pub(self) fn $setter_update(&self, r: StyleValueReferrer, val: $value_type, inherit: bool) {
            let changed = if inherit {
                let changed = !self.$name.inherit();
                self.$name.set_inherit(true);
                changed
            } else {
                let val = if r.is_absolute_or_relative() {
                    val
                } else {
                    $default_value
                };
                let changed = self.$name.equal(r, &val);
                self.$name.set(r, val);
                changed
            };
            if changed {
                let elem = self.element();
                if $layout_dirty {
                    elem.mark_layout_dirty();
                }
            }
        }
        pub fn $setter_set_inherit(&mut self) {
            let val = StyleValue::new($default_value_referrer, $default_value, true);
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val));
            self.element().mark_self_class_dirty();
        }
        #[inline]
        pub fn $setter_advanced(&mut self, r: StyleValueReferrer, val: $value_type) {
            let val = StyleValue::new(r, val, false);
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val));
            self.element().mark_self_class_dirty();
        }
        #[inline]
        pub fn $setter(&mut self, val: $value_type) {
            self.$setter_advanced(StyleValueReferrer::Absolute, val);
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

macro_rules! impl_parent_updated {
    ($($items:tt)*) => {
        impl ElementStyle {
            fn parent_updated(&mut self) {
                macro_rules! impl_parent_updated_item {
                    ($name:ident, $update_inherit:ident, $font_size_inherit:expr) => {
                        if self.$name.inherit() || ($font_size_inherit && self.$name.get_referrer().is_parent_relative()) {
                            Self::$update_inherit(self.node_mut());
                        }
                    }
                }
                if self.all_dirty.replace(true) {
                    return;
                }
                $($items)*
            }
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
        pub(self) fn apply_rule_from_class(style: &ElementStyle, name: &StyleName, value: &Box<dyn Any + Send>) {
            macro_rules! style_name {
                ($setter_inner: ident, $type: ty) => {
                    {
                        let style_value = value.downcast_ref::<StyleValue<$type>>().unwrap();
                        let (r, v) = style_value.get();
                        style.$setter_inner(r, v, style_value.inherit());
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
