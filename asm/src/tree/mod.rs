use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell, Ref, RefMut};

// tree node

pub trait TreeElem {
    fn associate_node(&mut self, _node: TreeNodeRc<Self>) where Self: Sized { }
}

pub struct TreeNode<T: TreeElem> {
    children: RefCell<Vec<TreeNodeRc<T>>>,
    parent: Cell<Option<TreeNodeWeak<T>>>,
    elem: RefCell<T>,
}

impl<T: TreeElem> TreeNode<T> {
    pub fn new(elem: T) -> Self {
        TreeNode {
            children: RefCell::new(vec!()),
            parent: Cell::new(None),
            elem: RefCell::new(elem),
        }
    }
}

// tree node ref

pub struct TreeNodeRc<T: TreeElem> {
    rc: Rc<TreeNode<T>>
}

pub struct TreeNodeWeak<T: TreeElem> {
    weak: Weak<TreeNode<T>>
}

impl<T: TreeElem> Clone for TreeNodeRc<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone()
        }
    }
}

impl<T: TreeElem> Clone for TreeNodeWeak<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone()
        }
    }
}

impl<T: TreeElem> From<Box<TreeNode<T>>> for TreeNodeRc<T> {
    fn from(boxed: Box<TreeNode<T>>) -> TreeNodeRc<T> {
        unsafe {
            TreeNodeRc {
                rc: Rc::from_raw(Box::into_raw(boxed))
            }
        }
    }
}

impl<T: TreeElem> TreeNodeWeak<T> {
    pub fn upgrade(&self) -> Option<TreeNodeRc<T>> {
        let opt = self.weak.upgrade();
        match opt {
            None => None,
            Some(x) => {
                Some(TreeNodeRc {
                    rc: x
                })
            }
        }
    }
    #[inline]
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        let a = a.weak.upgrade();
        let b = b.weak.upgrade();
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

impl<T: TreeElem> TreeNodeRc<T> {
    pub fn new(elem: T) -> Self {
        let mut ret = Self {
            rc: Rc::new(TreeNode::new(elem))
        };
        let ret_clone = ret.clone();
        ret.elem_mut().associate_node(ret_clone);
        ret
    }
    #[inline]
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Rc::ptr_eq(&a.rc, &b.rc)
    }
    pub fn downgrade(&self) -> TreeNodeWeak<T> {
        TreeNodeWeak {
            weak: Rc::downgrade(&self.rc)
        }
    }
    pub fn release_memory(&mut self) {
        let mut children = self.rc.children.borrow_mut();
        children.shrink_to_fit()
    }

    // content operators
    #[inline]
    pub fn elem_ref(&self) -> Ref<T> {
        self.rc.elem.borrow()
    }
    #[inline]
    pub fn elem_mut(&mut self) -> RefMut<T> {
        self.rc.elem.borrow_mut()
    }
    #[inline]
    pub fn ctx<F>(&mut self, f: &F) where F: Fn(&mut T) {
        f(&mut *self.rc.elem.borrow_mut())
    }
    #[inline]
    pub fn as_ptr(&mut self) -> *mut T {
        self.rc.elem.as_ptr()
    }

    // tree manipulation
    #[inline]
    pub fn len(&self) -> usize {
        let children = self.rc.children.borrow();
        children.len()
    }
    pub fn has_parent(&mut self) -> bool {
        let p = self.rc.parent.replace(None);
        let ret = match p {
            None => false,
            Some(ref x) => {
                match x.upgrade() {
                    None => false,
                    Some(ref _x) => true
                }
            }
        };
        self.rc.parent.set(p);
        ret
    }
    pub fn get_parent(&mut self) -> TreeNodeRc<T> {
        let p = self.rc.parent.replace(None);
        let ret = match p {
            None => panic!(),
            Some(ref x) => {
                (*x).clone()
            }
        };
        self.rc.parent.set(p);
        ret.upgrade().unwrap()
    }
    pub fn get_child(&mut self, index: usize) -> TreeNodeRc<T> {
        let children = self.rc.children.borrow_mut();
        children[index].clone()
    }
    pub fn append(&mut self, child: TreeNodeRc<T>) {
        child.rc.parent.set(Some(self.downgrade()));
        let mut children = self.rc.children.borrow_mut();
        children.push(child);
    }
    pub fn insert(&mut self, child: TreeNodeRc<T>, position: usize) {
        child.rc.parent.set(Some(self.downgrade()));
        let mut children = self.rc.children.borrow_mut();
        children.insert(position, child);
    }
    pub fn remove(&mut self, position: usize) -> TreeNodeRc<T> {
        let mut children = self.rc.children.borrow_mut();
        let child = children.remove(position);
        child.rc.parent.set(None);
        child
    }

    // iterator generators
    pub fn iter_children(&mut self) -> TreeNodeIter<T> {
        TreeNodeIter::new(self.clone())
    }
    pub fn dfs<F>(&mut self, search_type: TreeNodeSearchType, f: &mut F) -> bool where F: FnMut(&mut TreeNodeRc<T>) -> bool {
        let mut children = self.rc.children.borrow_mut();
        for mut child in children.iter_mut() {
            if search_type == TreeNodeSearchType::ChildrenFirst {
                if !child.dfs(search_type, f) {
                    return false;
                }
            }
            if !f(&mut child) {
                return false;
            }
            if search_type == TreeNodeSearchType::ChildrenLast {
                if !child.dfs(search_type, f) {
                    return false;
                }
            }
        }
        return true;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TreeNodeSearchType {
    NoChildren,
    ChildrenFirst,
    ChildrenLast,
}

// iterator

pub struct TreeNodeIter<T: TreeElem> {
    cur_index: usize,
    node_rc: TreeNodeRc<T>,
}

impl<T: TreeElem> TreeNodeIter<T> {
    fn new(node_rc: TreeNodeRc<T>) -> Self {
        TreeNodeIter {
            cur_index: 0,
            node_rc,
        }
    }
}

impl<T: TreeElem> Iterator for TreeNodeIter<T> {
    type Item = TreeNodeRc<T>;
    fn next(&mut self) -> Option<TreeNodeRc<T>> {
        if self.cur_index >= self.node_rc.len() {
            return None;
        }
        self.cur_index += 1;
        return Some(self.node_rc.get_child(self.cur_index - 1));
    }
}
