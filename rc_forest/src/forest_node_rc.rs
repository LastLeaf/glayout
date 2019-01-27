use std::ops::{Deref, DerefMut};
use std::cell::{Cell, UnsafeCell};
use std::rc::{Rc, Weak};
use super::*;

pub struct ForestNodeRc<T: ForestNodeContent> {
    status: Rc<ForestStatus>,
    forest_node: Rc<UnsafeCell<ForestNode<T>>>,
}

impl<T: ForestNodeContent> ForestNodeRc<T> {
    pub(crate) fn new(forest: &mut Forest<T>, forest_node: ForestNode<T>) -> Self {
        let ret = Self {
            status: forest.status.clone(),
            forest_node: Rc::new(UnsafeCell::new(forest_node)),
        };
        // TODO call associate_node
        ret
    }
    pub fn downgrade(&self) -> ForestNodeWeak<T> {
        ForestNodeWeak {
            status: self.status.clone(),
            forest_node: Cell::new(Rc::downgrade(&self.forest_node)),
        }
    }
    pub fn borrow<'a>(&self) -> ForestNodeRef<'a, T> {
        ForestNodeRef {
            status: ForestStatus::borrow(&self.status),
            forest_node: &*self.forest_node.get(),
        }
    }
    pub fn try_borrow<'a>(&self) -> Result<ForestNodeRef<'a, T>, ()> {
        match ForestStatus::try_borrow(&self.status) {
            Err(_) => Err(()),
            Ok(s) => {
                Ok(ForestNodeRef {
                    status: s,
                    forest_node: &*self.forest_node.get(),
                })
            }
        }
    }
    pub fn borrow_mut<'a>(&self) -> ForestNodeRefMut<'a, T> {
        ForestNodeRefMut {
            status: ForestStatus::borrow_mut(&self.status),
            forest_node: &mut *self.forest_node.get(),
        }
    }
    pub fn try_borrow_mut<'a>(&self) -> Result<ForestNodeRefMut<'a, T>, ()> {
        match ForestStatus::try_borrow_mut(&self.status) {
            Err(_) => Err(()),
            Ok(s) => {
                Ok(ForestNodeRefMut {
                    status: s,
                    forest_node: &mut *self.forest_node.get(),
                })
            }
        }
    }
}

pub struct ForestNodeRef<'a, T: ForestNodeContent> {
    status: ForestStatusRef,
    forest_node: &'a ForestNode<T>,
}

impl<'a, T: ForestNodeContent> Deref for ForestNodeRef<'a, T> {
    type Target = ForestNode<T>;
    fn deref(&self) -> &Self::Target {
        self.forest_node
    }
}

pub struct ForestNodeRefMut<'a, T: ForestNodeContent> {
    status: ForestStatusRefMut,
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
    status: Rc<ForestStatus>,
    forest_node: Cell<Weak<UnsafeCell<ForestNode<T>>>>,
}

impl<T: ForestNodeContent> ForestNodeWeak<T> {
    pub fn upgrade(&mut self) -> Option<ForestNodeRc<T>> {
        let option_rc = self.forest_node.get_mut().upgrade();
        match option_rc {
            None => None,
            Some(rc) => {
                Some(ForestNodeRc {
                    status: self.status.clone(),
                    forest_node: rc,
                })
            }
        }
    }
}
