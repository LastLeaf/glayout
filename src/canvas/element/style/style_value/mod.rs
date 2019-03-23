use std::fmt;
use std::mem;
use std::cell::Cell;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StyleValueReferrer {
    // global
    Absolute = 0x01,
    Auto = 0x02,

    // relative length
    RelativeToParentFontSize = 0x10,
    RelativeToParentSize = 0x11,
    RelativeToViewportFontSize = 0x18,
    RelativeToViewportWidth = 0x19,
    RelativeToViewportHeight = 0x1a,
}
pub use self::StyleValueReferrer::*;

const REFERRER_MASK: u8 = 0x1f;
const RELATIVE_BIT_MASK: u8 = 0x10;
const INHERIT: u8 = 0x40;
const DIRTY: u8 = 0x80;

impl StyleValueReferrer {
    pub fn is_absolute_or_relative(&self) -> bool {
        *self == StyleValueReferrer::Absolute || (*self as u8) & RELATIVE_BIT_MASK == RELATIVE_BIT_MASK
    }
    pub fn is_parent_relative(&self) -> bool {
        match self {
            StyleValueReferrer::RelativeToParentSize => true,
            StyleValueReferrer::RelativeToParentFontSize => true,
            _ => false
        }
    }
}

pub(crate) struct StyleValue<T: Clone> {
    cur_v: Cell<T>,
    v: Cell<T>,
    r: Cell<u8>,
}

impl<T: Clone + fmt::Debug> fmt::Debug for StyleValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({:?})", self.r.get(), unsafe { (*self.v.as_ptr()).clone() })
    }
}

impl<T: Clone> Clone for StyleValue<T> {
    fn clone(&self) -> Self {
        Self {
            cur_v: Cell::new(unsafe { (*self.cur_v.as_ptr()).clone() }),
            v: Cell::new(unsafe { (*self.v.as_ptr()).clone() }),
            r: Cell::new(self.r.get()),
        }
    }
}

impl<T: Clone> StyleValue<T> {
    pub(super) fn new(r: StyleValueReferrer, v: T, inherit: bool) -> Self {
        let ret = Self {
            cur_v: Cell::new(v.clone()),
            v: Cell::new(v),
            r: Cell::new(r as u8),
        };
        if inherit {
            ret.r.set(ret.r.get() | INHERIT | DIRTY);
        }
        ret
    }

    #[inline]
    pub(super) fn is_dirty(&self) -> bool {
        self.r.get() & DIRTY == DIRTY
    }
    #[inline]
    pub(super) fn clear_dirty(&self) {
        self.r.set(self.r.get() & !DIRTY)
    }
    #[inline]
    pub(super) fn get_and_mark_dirty(&self) -> bool {
        let ret = self.r.get() == self.r.get() | DIRTY;
        self.r.set(self.r.get() | DIRTY);
        ret
    }

    #[inline]
    pub(super) fn inherit(&self) -> bool {
        self.r.get() & INHERIT == INHERIT
    }
    #[inline]
    pub(super) fn set_inherit(&self, inherit: bool) {
        if self.inherit() == inherit {
            return;
        }
        self.r.set(self.r.get() & !INHERIT);
    }

    #[inline]
    pub(super) fn get_referrer(&self) -> StyleValueReferrer {
        let r = self.r.get() & REFERRER_MASK;
        unsafe { mem::transmute_copy(&r) }
    }
    #[inline]
    fn set_referrer(&self, r: StyleValueReferrer) {
        self.r.set((self.r.get() as u8 & !REFERRER_MASK) | r as u8);
    }

    #[inline]
    pub(super) fn get_value_ref(&mut self) -> &T {
        &*self.v.get_mut()
    }
    #[inline]
    pub(super) fn get_value_mut(&mut self) -> &mut T {
        self.v.get_mut()
    }
    #[inline]
    pub(super) fn get_current_value(&self) -> T {
        unsafe { (*self.cur_v.as_ptr()).clone() }
    }
    #[inline]
    pub(super) fn get(&self) -> (StyleValueReferrer, T) {
        (self.get_referrer(), unsafe { (*self.v.as_ptr()).clone() })
    }
    #[inline]
    pub(super) fn set(&self, r: StyleValueReferrer, v: T) {
        self.set_inherit(false);
        self.v.set(v);
        self.set_referrer(r);
    }
}
