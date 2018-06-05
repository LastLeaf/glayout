use std::rc::Rc;
use std::cell::{Cell, RefCell, Ref, RefMut};

// tree node

pub struct TreeNode<T> {
    children: RefCell<Vec<TreeNodeRc<T>>>,
    parent: Cell<Option<TreeNodeRc<T>>>,
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

impl<T> Clone for TreeNodeRc<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone()
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

impl<T> TreeNodeRc<T> {
    pub fn new(content: T) -> Self {
        Self {
            rc: Rc::new(TreeNode::new(content))
        }
    }
    pub fn release_memory(&mut self) {
        let mut children = self.rc.children.borrow_mut();
        children.shrink_to_fit()
    }
    pub fn get(&self) -> Ref<T> {
        self.rc.content.borrow()
    }
    pub fn get_mut(&mut self) -> RefMut<T> {
        self.rc.content.borrow_mut()
    }
    pub fn as_ptr(&mut self) -> *mut T {
        self.rc.content.as_ptr()
    }
    pub fn len(&self) -> usize {
        let children = self.rc.children.borrow();
        children.len()
    }
    pub fn has_parent(&mut self) -> bool {
        let p = self.rc.parent.replace(None);
        let ret = match p {
            None => false,
            Some(ref _x) => true
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
        ret
    }
    pub fn get_child(&mut self, index: usize) -> TreeNodeRc<T> {
        let children = self.rc.children.borrow_mut();
        children[index].clone()
    }
    pub fn append(&mut self, child: TreeNodeRc<T>) {
        child.rc.parent.set(Some((*self).clone()));
        let mut children = self.rc.children.borrow_mut();
        children.push(child);
    }
    pub fn insert(&mut self, child: TreeNodeRc<T>, position: usize) {
        child.rc.parent.set(Some((*self).clone()));
        let mut children = self.rc.children.borrow_mut();
        children.insert(position, child);
    }
    pub fn remove(&mut self, position: usize) -> TreeNodeRc<T> {
        let mut children = self.rc.children.borrow_mut();
        let child = children.remove(position);
        child.rc.parent.set(None);
        child
    }
    pub fn iter_children(&mut self) -> TreeNodeIter<T> {
        TreeNodeIter::new(self.clone(), TreeNodeIterSearchType::NoChildren)
    }
    pub fn dfs(&mut self, search_type: TreeNodeIterSearchType) -> TreeNodeIter<T> {
        TreeNodeIter::new(self.clone(), search_type)
    }
}

// iterator

pub enum TreeNodeIterSearchType {
    NoChildren,
    ChildrenFirst,
    ChildrenLast,
}

pub struct TreeNodeIter<T> {
    search_type: TreeNodeIterSearchType,
    cur_index: usize,
    cur_rc: TreeNodeRc<T>,
    index_stack: Vec<usize>,
    rc_stack: Vec<TreeNodeRc<T>>,
}

impl<T> TreeNodeIter<T> {
    fn new(rc: TreeNodeRc<T>, search_type: TreeNodeIterSearchType) -> Self {
        TreeNodeIter {
            search_type,
            cur_index: 0,
            cur_rc: rc,
            index_stack: vec![],
            rc_stack: vec![],
        }
    }
}

impl<T> Iterator for TreeNodeIter<T> {
    type Item = TreeNodeRc<T>;
    fn next(&mut self) -> Option<TreeNodeRc<T>> {
        unimplemented!();
        // self.cur_index += 1;
        // let node = self.cur_rc.rc;
        // if self.cur_index >= node.children.len() {
        //     if self.rc_stack.len() > 0 {
        //         self.cur_rc = self.rc_stack.pop();
        //         self.cur_index = self.index_stack.pop();
        //     }
        // } else {
        //     // initial state
        //     self.index_stack.push(0)
        // }
    }
}
