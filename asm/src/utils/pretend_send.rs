use std::ops::{Deref, DerefMut};
use std::thread;

/// Limit T to be used in only one thread. Deref in other threads would `panic!``. Useful for statics.
pub struct PretendSend<T> {
    thread_id: thread::ThreadId,
    content: T,
}

unsafe impl<T> Send for PretendSend<T> { }
unsafe impl<T> Sync for PretendSend<T> { }

impl<T> Deref for PretendSend<T> {
    type Target = T;
    fn deref(&self) -> &T {
        if thread::current().id() != self.thread_id {
            panic!("PretendSend can only be used in the thread which creates it.");
        }
        &self.content
    }
}

impl<T> DerefMut for PretendSend<T> {
    fn deref_mut(&mut self) -> &mut T {
        if thread::current().id() != self.thread_id {
            panic!("PretendSend can only be used in the thread which creates it.");
        }
        &mut self.content
    }
}

impl<T> PretendSend<T> {
    pub fn new(content: T) -> Self {
        PretendSend {
            thread_id: thread::current().id(),
            content,
        }
    }
    #[inline]
    pub fn match_thread_id(&self, thread_id: thread::ThreadId) {
        if thread_id != self.thread_id {
            panic!("PretendSend can only be used in the thread which creates it.");
        }
    }
}
