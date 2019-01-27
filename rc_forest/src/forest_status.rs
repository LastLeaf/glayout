use std::rc::Rc;
use std::cell::UnsafeCell;

pub(crate) struct ForestStatus {
    is_mut: UnsafeCell<bool>,
    count: UnsafeCell<usize>,
}

impl ForestStatus {
    pub(crate) fn new() -> Self {
        Self {
            is_mut: UnsafeCell::new(false),
            count: UnsafeCell::new(0),
        }
    }
    #[inline]
    pub(crate) fn borrow(s: &Rc<Self>) -> ForestStatusRef {
        unsafe {
            if *s.is_mut.get() || *s.count.get() > 0 { panic!("Forest has been borrowed") };
            *s.count.get() += 1;
        }
        ForestStatusRef::new(s.clone())
    }
    #[inline]
    pub(crate) fn try_borrow(s: &Rc<Self>) -> Result<ForestStatusRef, ()> {
        unsafe {
            if *s.is_mut.get() || *s.count.get() > 0 {
                return Err(());
            }
            *s.count.get() += 1;
        }
        Ok(ForestStatusRef::new(s.clone()))
    }
    #[inline]
    pub(crate) fn borrow_mut(s: &Rc<Self>) -> ForestStatusRefMut {
        unsafe {
            if *s.count.get() > 0 { panic!("Forest has been borrowed") };
            *s.is_mut.get() = true;
            *s.count.get() += 1;
        }
        ForestStatusRefMut::new(s.clone())
    }
    #[inline]
    pub(crate) fn try_borrow_mut(s: &Rc<Self>) -> Result<ForestStatusRefMut, ()> {
        unsafe {
            if *s.count.get() > 0 {
                return Err(());
            }
            *s.is_mut.get() = true;
            *s.count.get() += 1;
        }
        Ok(ForestStatusRefMut::new(s.clone()))
    }
}

pub(crate) struct ForestStatusRef {
    status: Rc<ForestStatus>,
}

impl ForestStatusRef {
    pub(crate) fn new(status: Rc<ForestStatus>) -> Self {
        Self {
            status
        }
    }
}

impl Drop for ForestStatusRef {
    fn drop(&mut self) {
        let c = self.status.count.get();
        *c -= 1;
        if *c == 0 {
            *self.status.is_mut.get() = false;
        }
    }
}

pub(crate) struct ForestStatusRefMut {
    status: Rc<ForestStatus>,
}

impl ForestStatusRefMut {
    pub(crate) fn new(status: Rc<ForestStatus>) -> Self {
        Self {
            status
        }
    }
}

impl<'a> Drop for ForestStatusRefMut {
    fn drop(&mut self) {
        let c = self.status.count.get();
        *c -= 1;
        if *c == 0 {
            *self.status.is_mut.get() = false;
        }
    }
}
