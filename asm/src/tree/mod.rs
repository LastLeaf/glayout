use std::ptr;
use std::rc::Rc;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// tree node

pub struct TreeNode<T> {
    children: Vec<*mut TreeNode<T>>,
    parent: *mut TreeNode<T>,
    content: T
}

unsafe impl<T> Send for TreeNode<T> { }
unsafe impl<T> Sync for TreeNode<T> { }

impl<T> TreeNode<T> {
    pub fn new(content: T) -> Self {
        TreeNode {
            children: vec!(),
            parent: ptr::null_mut(),
            content,
        }
    }
}

// tree node ref

pub struct TreeNodeCtx<T> {
    pointer: *mut TreeNode<T>,
    // this ref is neither Send nor Sync, so add an Rc here
    phantom_data: PhantomData<Rc<T>>
}

impl<T> From<*mut TreeNode<T>> for TreeNodeCtx<T> {
    fn from(pointer: *mut TreeNode<T>) -> TreeNodeCtx<T> {
        TreeNodeCtx {
            pointer,
            phantom_data: PhantomData
        }
    }
}

impl<'a, T> From<&'a mut TreeNode<T>> for TreeNodeCtx<T> {
    fn from(ref_val: &'a mut TreeNode<T>) -> TreeNodeCtx<T> {
        TreeNodeCtx {
            pointer: ref_val as *mut TreeNode<T>,
            phantom_data: PhantomData
        }
    }
}

impl<T> TreeNodeCtx<T> {
    pub fn release_memory(&mut self) {
        unsafe {
            let node = &mut *self.pointer;
            node.children.shrink_to_fit()
        }
    }
    pub fn len(&self) {
        unsafe {
            let node = &mut *self.pointer;
            node.children.len();
        }
    }
    pub fn get_parent(&mut self) -> Option<TreeNodeCtx<T>> {
        unsafe {
            let node = &mut *self.pointer;
            if node.parent.is_null() {
                return None
            }
            Some(TreeNodeCtx::from(node.parent))
        }
    }
    pub fn get_child(&mut self, index: usize) -> Option<TreeNodeCtx<T>> {
        unsafe {
            let node = &mut *self.pointer;
            Some(TreeNodeCtx::from(node.children[index]))
        }
    }
    pub fn append(&mut self, mut child: Box<TreeNode<T>>) {
        unsafe {
            let node = &mut *self.pointer;
            child.parent = self.pointer;
            node.children.push(Box::into_raw(child))
        }
    }
    pub fn insert(&mut self, mut child: Box<TreeNode<T>>, position: usize) {
        unsafe {
            let node = &mut *self.pointer;
            child.parent = self.pointer;
            node.children.insert(position, Box::into_raw(child))
        }
    }
    pub fn remove(&mut self, position: usize) -> Box<TreeNode<T>> {
        unsafe {
            let node = &mut *self.pointer;
            let child = node.children.remove(position);
            (*child).parent = ptr::null_mut();
            Box::from_raw(child)
        }
    }
}

impl<T> Deref for TreeNodeCtx<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            let node = &mut *self.pointer;
            &node.content
        }
    }
}

impl<T> DerefMut for TreeNodeCtx<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            let node = &mut *self.pointer;
            &mut node.content
        }
    }
}

// TODO impl iterator
