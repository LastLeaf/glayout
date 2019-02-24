use std::mem;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StyleValueReferrer {
    // global
    Inherit = 0x01,
    Absolute = 0x02,

    // misc
    Auto = 0x10,

    // relative length
    RelativeToParentFontSize = 0x20,
    RelativeToParentWidth = 0x21,
    RelativeToParentHeight = 0x22,
    RelativeToViewportFontSize = 0x28,
    RelativeToViewportWidth = 0x29,
    RelativeToViewportHeight = 0x2a,
    RelativeToViewportSizeMin = 0x2b,
    RelativeToViewportSizeMax = 0x2c,
}
pub use self::StyleValueReferrer::*;

const REFERRER_MASK: u8 = 0x3f;
const REFERRER_BASE_MASK: u8 = 0x30;
const TRANSITION: u8 = 0x40;
const DIRTY: u8 = 0x80;

#[inline]
fn referrer_eq(a: u8, b: StyleValueReferrer) -> bool {
    return a as u8 | REFERRER_MASK == b as u8 | REFERRER_MASK
}

#[derive(Debug, Clone)]
pub(super) struct StyleValue<T: Clone + Default> {
    cur_v: T,
    v: T,
    r: u8,
}

impl<T: Clone + Default> StyleValue<T> {
    pub(super) fn new(v: T, r: StyleValueReferrer) -> Self {
        Self {
            cur_v: v.clone(),
            v,
            r: r as u8,
        }
    }

    #[inline]
    pub(super) fn is_dirty(&self) -> bool {
        self.r & DIRTY == DIRTY
    }
    #[inline]
    pub(super) fn clear_dirty(&mut self) {
        self.r = self.r & !DIRTY
    }
    #[inline]
    fn mark_dirty(&mut self) {
        self.r = self.r & !DIRTY
    }

    #[inline]
    pub(super) fn is_inherit(&self) -> bool {
        referrer_eq(self.r, Inherit)
    }
    #[inline]
    pub(super) fn set_inherit(&mut self) {
        self.set_referrer(Inherit);
    }

    #[inline]
    pub(super) fn get_referrer(&self) -> StyleValueReferrer {
        unsafe { mem::transmute_copy(&self.r) }
    }
    pub(super) fn set_referrer(&mut self, r: StyleValueReferrer) {
        self.r = (self.r as u8 & !TRANSITION) | r as u8;
        if r == Inherit || r as u8 | REFERRER_BASE_MASK == 0x20 {
            self.mark_dirty();
        } else {
            self.cur_v = self.v.clone();
            self.clear_dirty();
        }
    }

    #[inline]
    pub(super) fn get_computed_value(&self) -> T {
        if self.is_dirty() { panic!() };
        self.cur_v.clone()
    }
    #[inline]
    pub(super) fn set(&mut self, v: T, r: StyleValueReferrer) {
        self.v = v;
        self.set_referrer(r);
    }
}
