use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};

// tree node

pub trait TreeElem: Clone {
    #[inline]
    fn associate_node(&self, _node: TreeNodeWeak<Self>) where Self: Sized { }
    #[inline]
    fn parent_node_changed(&self, _parent_node: Option<TreeNodeRc<Self>>) where Self: Sized { }
}

pub struct TreeNode<T: TreeElem> {
    children: RefCell<Vec<TreeNodeRc<T>>>,
    parent: Cell<Option<TreeNodeWeak<T>>>,
    elem: T,
}

impl<T: TreeElem> TreeNode<T> {
    pub fn new(elem: T) -> Self {
        TreeNode {
            children: RefCell::new(vec!()),
            parent: Cell::new(None),
            elem,
        }
    }
}

impl<T: TreeElem> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            children: RefCell::new(vec!()),
            parent: Cell::new(None),
            elem: self.elem.clone()
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
        let ret = Self {
            rc: Rc::new(TreeNode::new(elem))
        };
        let ret_clone = ret.downgrade();
        ret.rc.elem.associate_node(ret_clone);
        ret
    }
    pub fn clone_node(&self) -> Self {
        let ret = Self {
            rc: Rc::new((*self.rc).clone())
        };
        let ret_clone = ret.downgrade();
        ret.rc.elem.associate_node(ret_clone);
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
    pub fn shrink_memory(&mut self) {
        let mut children = self.rc.children.borrow_mut();
        children.shrink_to_fit()
    }
    pub fn into_ptr(n: Self) -> *const TreeNode<T> {
        Rc::into_raw(n.rc)
    }
    pub unsafe fn from_ptr(ptr: *const TreeNode<T>, need_clone: bool) -> Self {
        let rc = Rc::from_raw(ptr);
        if need_clone {
            Rc::into_raw(rc.clone());
        }
        Self {
            rc
        }
    }

    // content operators
    #[inline]
    pub fn elem(&self) -> &T {
        &self.rc.elem
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
    pub fn parent(&self) -> Option<TreeNodeRc<T>> {
        let p = self.rc.parent.replace(None);
        let ret = match p {
            None => None,
            Some(ref x) => {
                x.upgrade()
            }
        };
        self.rc.parent.set(p);
        ret
    }
    pub fn child(&self, index: usize) -> TreeNodeRc<T> {
        let children = self.rc.children.borrow_mut();
        children[index].clone()
    }
    pub fn append(&mut self, child: TreeNodeRc<T>) {
        child.replace_from_old_parent(Some(self.downgrade()));
        child.elem().parent_node_changed(Some(self.clone()));
        let mut children = self.rc.children.borrow_mut();
        children.push(child.clone());
    }
    pub fn insert(&mut self, child: TreeNodeRc<T>, position: usize) {
        child.replace_from_old_parent(Some(self.downgrade()));
        child.elem().parent_node_changed(Some(self.clone()));
        let mut children = self.rc.children.borrow_mut();
        children.insert(position, child.clone());
    }
    pub fn remove(&mut self, position: usize) -> TreeNodeRc<T> {
        let mut children = self.rc.children.borrow_mut();
        let child = children.remove(position);
        child.rc.parent.set(None);
        child.elem().parent_node_changed(None);
        child
    }
    pub fn replace(&mut self, new_child: TreeNodeRc<T>, position: usize) -> TreeNodeRc<T> {
        let mut children = self.rc.children.borrow_mut();
        let child = children[position].clone();
        children[position] = new_child.clone();
        child.rc.parent.set(None);
        child.elem().parent_node_changed(None);
        new_child.replace_from_old_parent(Some(self.downgrade()));
        new_child.elem().parent_node_changed(Some(self.clone()));
        child
    }
    fn replace_from_old_parent(&self, new_parent: Option<TreeNodeWeak<T>>) {
        let prev_parent = self.rc.parent.replace(new_parent);
        match prev_parent {
            Some(x) => {
                let mut parent = x.upgrade().unwrap();
                let i = parent.find_child_position(&self).unwrap();
                parent.remove(i);
            },
            None => {}
        }
    }
    pub fn find_child_position(&self, child: &TreeNodeRc<T>) -> Option<usize> {
        for i in 0..self.len() {
            let c = self.child(i);
            if TreeNodeRc::ptr_eq(child, &c) {
                return Some(i);
            }
        }
        None
    }
    pub fn splice(&mut self, position: usize, removes: usize, mut other_children_parent: Option<TreeNodeRc<T>>) {
        let mut children = self.rc.children.borrow_mut();
        let inserts = match other_children_parent {
            None => { vec![] },
            Some(ref mut x) => {
                let mut children = vec![];
                children.append(&mut x.rc.children.borrow_mut());
                for child in children.iter() {
                    child.replace_from_old_parent(Some(self.downgrade()));
                    child.elem().parent_node_changed(Some(self.clone()));
                }
                children
            }
        };
        children.splice(position..(position + removes), inserts);
    }

    // iterator generators
    pub fn iter_children(&self) -> TreeNodeIter<T> {
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
    cur_index_start: usize,
    cur_index_end: usize,
    node_rc: TreeNodeRc<T>,
}

impl<T: TreeElem> TreeNodeIter<T> {
    fn new(node_rc: TreeNodeRc<T>) -> Self {
        TreeNodeIter {
            cur_index_start: 0,
            cur_index_end: node_rc.len(),
            node_rc,
        }
    }
}

impl<T: TreeElem> Iterator for TreeNodeIter<T> {
    type Item = TreeNodeRc<T>;
    fn next(&mut self) -> Option<TreeNodeRc<T>> {
        if self.cur_index_start >= self.cur_index_end {
            return None;
        }
        self.cur_index_start += 1;
        return Some(self.node_rc.child(self.cur_index_start - 1));
    }
}

impl<T: TreeElem> DoubleEndedIterator for TreeNodeIter<T> {
    fn next_back(&mut self) -> Option<TreeNodeRc<T>> {
        if self.cur_index_start >= self.cur_index_end {
            return None;
        }
        self.cur_index_end -= 1;
        return Some(self.node_rc.child(self.cur_index_end));
    }
}
