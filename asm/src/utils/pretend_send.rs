#![macro_use]

use std::ops::{Deref, DerefMut};

// NOTE PretendSend is only safe to use in single-threaded env.

pub struct PretendSend<T> {
    content: T
}

unsafe impl<T> Send for PretendSend<T> { }
unsafe impl<T> Sync for PretendSend<T> { }

impl<T> Deref for PretendSend<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.content
    }
}

impl<T> DerefMut for PretendSend<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.content
    }
}

impl<T> PretendSend<T> {
    pub fn new(content: T) -> Self {
        PretendSend {
            content
        }
    }
}
