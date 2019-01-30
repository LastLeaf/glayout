use super::*;

pub struct ForestNodeSelf<T: ForestNodeContent> {
    weak: ForestNodeWeak<T>,
    content_ptr: *const T,
}

impl<T: ForestNodeContent> ForestNodeSelf<T> {
    pub(crate) fn new(weak: ForestNodeWeak<T>, content: &T) -> Self {
        Self {
            weak,
            content_ptr: content as *const T,
        }
    }
    #[inline]
    pub fn rc(&self) -> ForestNodeRc<T> {
        self.weak.upgrade().unwrap()
    }
    pub fn deref_by<'a, 'b>(&'b self, content: &'a T) -> &'a ForestNode<T> {
        if content as *const T != self.content_ptr {
            panic!("ForestNodeSelf can only be deref by corresponding ForestNodeContent");
        }
        unsafe { self.weak.upgrade().unwrap().forest_node() }
    }
    pub fn deref_mut_by<'a, 'b>(&'b self, content: &'a mut T) -> &'a mut ForestNode<T> {
        if content as *const T != self.content_ptr {
            panic!("ForestNodeSelf can only be deref mut by corresponding ForestNodeContent");
        }
        unsafe { self.weak.upgrade().unwrap().forest_node_mut() }
    }
}

impl<T: ForestNodeContent> Clone for ForestNodeSelf<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
            content_ptr: self.content_ptr,
        }
    }
}
