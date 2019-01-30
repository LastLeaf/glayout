use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;
use std::rc::{Rc, Weak};
use super::*;

pub type ForestNodePtr<T> = *const UnsafeCell<ForestNode<T>>;

pub struct ForestNodeRc<T: ForestNodeContent> {
    forest_node: Rc<UnsafeCell<ForestNode<T>>>,
}

impl<T: ForestNodeContent> ForestNodeRc<T> {
    #[inline]
    pub(crate) unsafe fn create_in_context(context: Rc<ForestContext>, content: T) -> Self {
        let forest_node = ForestNode::new(context, content);
        let ret = Self {
            forest_node: Rc::new(UnsafeCell::new(forest_node)),
        };
        let n = ret.forest_node_mut();
        n.set_self_weak(ret.downgrade());
        ret
    }
    pub fn new(forest: &mut Forest<T>, content: T) -> Self {
        ForestContext::borrow_mut(&forest.context);
        unsafe {
            Self::create_in_context(forest.context.clone(), content)
        }
    }

    #[inline]
    pub unsafe fn forest_node<'a>(&self) -> &'a ForestNode<T> {
        &*self.forest_node.get()
    }
    #[inline]
    pub unsafe fn forest_node_mut<'a>(&self) -> &'a mut ForestNode<T> {
        &mut *self.forest_node.get()
    }
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Rc::ptr_eq(&a.forest_node, &b.forest_node)
    }
    pub fn downgrade(&self) -> ForestNodeWeak<T> {
        ForestNodeWeak {
            forest_node: Rc::downgrade(&self.forest_node),
        }
    }
    #[inline]
    pub fn into_ptr(self) -> ForestNodePtr<T> {
        Rc::into_raw(self.forest_node)
    }
    #[inline]
    pub unsafe fn from_ptr(ptr: ForestNodePtr<T>, need_clone: bool) -> Self {
        let ret = Self {
            forest_node: Rc::from_raw(ptr),
        };
        if need_clone {
            Rc::into_raw(ret.forest_node.clone());
        }
        ret
    }

    pub fn borrow<'a>(&self) -> ForestNodeRef<'a, T> {
        ForestNodeRef {
            _status: ForestContext::borrow(unsafe { self.forest_node() }.context()),
            forest_node: unsafe { self.forest_node() },
        }
    }
    pub fn try_borrow<'a>(&self) -> Result<ForestNodeRef<'a, T>, ()> {
        match ForestContext::try_borrow(unsafe { self.forest_node() }.context()) {
            Err(_) => Err(()),
            Ok(s) => {
                Ok(ForestNodeRef {
                    _status: s,
                    forest_node: unsafe { self.forest_node() },
                })
            }
        }
    }
    pub fn borrow_mut<'a>(&self) -> ForestNodeRefMut<'a, T> {
        ForestNodeRefMut {
            _status: ForestContext::borrow_mut(unsafe { self.forest_node() }.context()),
            forest_node: unsafe { self.forest_node_mut() },
        }
    }
    pub fn try_borrow_mut<'a>(&self) -> Result<ForestNodeRefMut<'a, T>, ()> {
        match ForestContext::try_borrow_mut(unsafe { self.forest_node() }.context()) {
            Err(_) => Err(()),
            Ok(s) => {
                Ok(ForestNodeRefMut {
                    _status: s,
                    forest_node: unsafe { self.forest_node_mut() },
                })
            }
        }
    }
    pub fn deref_with<'a, 'b>(&'b self, source: &'a ForestNode<T>) -> &'a ForestNode<T> {
        if !Rc::ptr_eq(source.context(), unsafe { self.forest_node() }.context()) {
            panic!("A ForestNode can only be deref by another ForestNode in the same Forest");
        }
        unsafe { self.forest_node() }
    }
    pub fn deref_mut_with<'a, 'b>(&'b self, source: &'a mut ForestNode<T>) -> &'a mut ForestNode<T> {
        if !Rc::ptr_eq(source.context(), unsafe { self.forest_node() }.context()) {
            panic!("A ForestNode can only be deref mut by another ForestNode in the same Forest");
        }
        unsafe { self.forest_node_mut() }
    }
}

impl<T: ForestNodeContent> Clone for ForestNodeRc<T> {
    fn clone(&self) -> Self {
        Self {
            forest_node: self.forest_node.clone(),
        }
    }
}

impl<T: ForestNodeContent + Clone> ForestNodeRc<T> {
    #[inline]
    pub fn clone_node_with(&self, other: &mut ForestNode<T>) -> Self {
        let ret = Self {
            forest_node: Rc::new(UnsafeCell::new(self.deref_with(other).clone_node()))
        };
        let n = unsafe { ret.forest_node_mut() };
        n.set_self_weak(ret.downgrade());
        ret
    }
}


pub struct ForestNodeRef<'a, T: ForestNodeContent> {
    _status: ForestContextRef,
    forest_node: &'a ForestNode<T>,
}

impl<'a, T: ForestNodeContent> Deref for ForestNodeRef<'a, T> {
    type Target = ForestNode<T>;
    fn deref(&self) -> &Self::Target {
        self.forest_node
    }
}


pub struct ForestNodeRefMut<'a, T: ForestNodeContent> {
    _status: ForestContextRefMut,
    forest_node: &'a mut ForestNode<T>,
}

impl<'a, T: ForestNodeContent> Deref for ForestNodeRefMut<'a, T> {
    type Target = ForestNode<T>;
    fn deref(&self) -> &Self::Target {
        self.forest_node
    }
}

impl<'a, T: ForestNodeContent> DerefMut for ForestNodeRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.forest_node
    }
}


pub struct ForestNodeWeak<T: ForestNodeContent> {
    forest_node: Weak<UnsafeCell<ForestNode<T>>>,
}

impl<T: ForestNodeContent> ForestNodeWeak<T> {
    pub fn upgrade(&self) -> Option<ForestNodeRc<T>> {
        let option_rc = self.forest_node.upgrade();
        match option_rc {
            None => None,
            Some(rc) => {
                Some(ForestNodeRc {
                    forest_node: rc,
                })
            }
        }
    }
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        let a = a.forest_node.upgrade();
        let b = b.forest_node.upgrade();
        match a {
            None => {
                b.is_none()
            },
            Some(ref ai) => {
                match b {
                    None => {
                        false
                    },
                    Some(ref bi) => {
                        Rc::ptr_eq(ai, bi)
                    }
                }
            }
        }
    }
}

impl<T: ForestNodeContent> Clone for ForestNodeWeak<T> {
    fn clone(&self) -> Self {
        Self {
            forest_node: self.forest_node.clone(),
        }
    }
}
