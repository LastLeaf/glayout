use std::rc::Rc;
use std::cell::UnsafeCell;

pub(crate) struct ForestContext {
    is_mut: UnsafeCell<bool>,
    count: UnsafeCell<usize>,
}

impl ForestContext {
    pub(crate) fn new() -> Self {
        Self {
            is_mut: UnsafeCell::new(false),
            count: UnsafeCell::new(0),
        }
    }
    #[inline]
    pub(crate) fn borrow(s: &Rc<Self>) -> ForestContextRef {
        unsafe {
            if *s.is_mut.get() || *s.count.get() > 0 { panic!("Forest has been borrowed") };
            *s.count.get() += 1;
        }
        ForestContextRef::new(s.clone())
    }
    #[inline]
    pub(crate) fn try_borrow(s: &Rc<Self>) -> Result<ForestContextRef, ()> {
        unsafe {
            if *s.is_mut.get() || *s.count.get() > 0 {
                return Err(());
            }
            *s.count.get() += 1;
        }
        Ok(ForestContextRef::new(s.clone()))
    }
    #[inline]
    pub(crate) fn borrow_mut(s: &Rc<Self>) -> ForestContextRefMut {
        unsafe {
            if *s.count.get() > 0 { panic!("Forest has been borrowed") };
            *s.is_mut.get() = true;
            *s.count.get() += 1;
        }
        ForestContextRefMut::new(s.clone())
    }
    #[inline]
    pub(crate) fn try_borrow_mut(s: &Rc<Self>) -> Result<ForestContextRefMut, ()> {
        unsafe {
            if *s.count.get() > 0 {
                return Err(());
            }
            *s.is_mut.get() = true;
            *s.count.get() += 1;
        }
        Ok(ForestContextRefMut::new(s.clone()))
    }
}

pub(crate) struct ForestContextRef {
    context: Rc<ForestContext>,
}

impl ForestContextRef {
    pub(crate) fn new(context: Rc<ForestContext>) -> Self {
        Self {
            context
        }
    }
}

impl Drop for ForestContextRef {
    fn drop(&mut self) {
        let c = self.context.count.get();
        unsafe {
            *c -= 1;
            if *c == 0 {
                *self.context.is_mut.get() = false;
            }
        }
    }
}

pub(crate) struct ForestContextRefMut {
    context: Rc<ForestContext>,
}

impl ForestContextRefMut {
    pub(crate) fn new(context: Rc<ForestContext>) -> Self {
        Self {
            context
        }
    }
}

impl<'a> Drop for ForestContextRefMut {
    fn drop(&mut self) {
        let c = self.context.count.get();
        unsafe {
            *c -= 1;
            if *c == 0 {
                *self.context.is_mut.get() = false;
            }
        }
    }
}
