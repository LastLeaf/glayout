use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell, Ref, RefMut};

// tree node

pub struct TreeNode<T> {
    children: RefCell<Vec<TreeNodeRc<T>>>,
    parent: Cell<Option<TreeNodeWeak<T>>>,
    content: RefCell<T>
}

impl<T> TreeNode<T> {
    pub fn new(content: T) -> Self {
        TreeNode {
            children: RefCell::new(vec!()),
            parent: Cell::new(None),
            content: RefCell::new(content),
        }
    }
}

// tree node ref

pub struct TreeNodeRc<T> {
    rc: Rc<TreeNode<T>>
}

pub struct TreeNodeWeak<T> {
    weak: Weak<TreeNode<T>>
}

impl<T> Clone for TreeNodeRc<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone()
        }
    }
}

impl<T> Clone for TreeNodeWeak<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone()
        }
    }
}

impl<T> From<Box<TreeNode<T>>> for TreeNodeRc<T> {
    fn from(boxed: Box<TreeNode<T>>) -> TreeNodeRc<T> {
        unsafe {
            TreeNodeRc {
                rc: Rc::from_raw(Box::into_raw(boxed))
            }
        }
    }
}

impl<T> TreeNodeWeak<T> {
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
}

impl<T> TreeNodeRc<T> {
    pub fn new(content: T) -> Self {
        Self {
            rc: Rc::new(TreeNode::new(content))
        }
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
    pub fn get(&self) -> Ref<T> {
        self.rc.content.borrow()
    }
    pub fn get_mut(&mut self) -> RefMut<T> {
        self.rc.content.borrow_mut()
    }
    pub fn ctx<F>(&mut self, f: &F) where F: Fn(&mut T) {
        f(&mut *self.rc.content.borrow_mut())
    }
    pub fn as_ptr(&mut self) -> *mut T {
        self.rc.content.as_ptr()
    }

    // tree manipulation
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
    pub fn dfs<F>(&mut self, search_type: TreeNodeSearchType, f: &F) where F: Fn(&mut T) {
        let mut children = self.rc.children.borrow_mut();
        for child in children.iter_mut() {
            if search_type == TreeNodeSearchType::ChildrenFirst {
                child.dfs(search_type, f);
            }
            f(&mut *child.rc.content.borrow_mut());
            if search_type == TreeNodeSearchType::ChildrenLast {
                child.dfs(search_type, f);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TreeNodeSearchType {
    NoChildren,
    ChildrenFirst,
    ChildrenLast,
}

// iterator

pub struct TreeNodeIter<T> {
    cur_index: usize,
    node_rc: TreeNodeRc<T>,
}

impl<T> TreeNodeIter<T> {
    fn new(node_rc: TreeNodeRc<T>) -> Self {
        TreeNodeIter {
            cur_index: 0,
            node_rc,
        }
    }
}

impl<T> Iterator for TreeNodeIter<T> {
    type Item = TreeNodeRc<T>;
    fn next(&mut self) -> Option<TreeNodeRc<T>> {
        if self.cur_index >= self.node_rc.len() {
            return None;
        }
        self.cur_index += 1;
        return Some(self.node_rc.get_child(self.cur_index - 1));
    }
}
