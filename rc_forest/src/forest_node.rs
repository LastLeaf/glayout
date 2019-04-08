use std::ops::{Deref, DerefMut, BitXor, Index, IndexMut, Range};
use std::slice::Iter;
use super::*;

pub struct ForestNode<T: ForestNodeContent> {
    context: Rc<ForestContext>,
    self_weak: Option<ForestNodeWeak<T>>,
    parent: Option<ForestNodeWeak<T>>,
    children: Vec<ForestNodeRc<T>>,
    content: T,
}

impl<T: ForestNodeContent> ForestNode<T> {
    pub(crate) unsafe fn new(context: Rc<ForestContext>, content: T) -> Self {
        let ret = Self {
            context,
            self_weak: None,
            parent: None,
            children: vec![],
            content,
        };
        ret
    }
    pub fn create_another(&mut self, content: T) -> ForestNodeRc<T> {
        unsafe {
            ForestNodeRc::create_in_context(self.context.clone(), content)
        }
    }

    #[inline]
    pub(crate) fn set_self_weak(&mut self, weak: ForestNodeWeak<T>) {
        self.self_weak = Some(weak.clone());
        let ns = ForestNodeSelf::new(weak, &self.content);
        self.content.associate_node(ns);
    }
    #[inline]
    pub(crate) fn context(&self) -> &Rc<ForestContext> {
        &self.context
    }
    #[inline]
    pub fn another<'a, 'b>(&'a self, another_rc: &'b ForestNodeRc<T>) -> &'a Self {
        another_rc.deref_with(self)
    }
    #[inline]
    pub fn another_mut<'a, 'b>(&'a mut self, another_rc: &'b ForestNodeRc<T>) -> &'a mut Self {
        another_rc.deref_mut_with(self)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.children.len()
    }
    #[inline]
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    #[inline]
    pub fn parent<'a>(&'a self) -> Option<&'a Self> {
        match self.parent {
            None => None,
            Some(ref p) => {
                match p.upgrade() {
                    None => None,
                    Some(p) => Some(p.deref_with(self))
                }
            },
        }
    }
    #[inline]
    pub fn parent_mut<'a>(&'a mut self) -> Option<&'a mut Self> {
        match self.parent {
            None => None,
            Some(ref p) => {
                match p.upgrade() {
                    None => None,
                    Some(p) => Some(p.deref_mut_with(self))
                }
            }
        }
    }
    #[inline]
    pub fn child<'a>(&'a self, index: usize) -> Option<&'a Self> {
        self.children.get(index).map(|x| x.deref_with(self))
    }
    #[inline]
    pub fn child_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Self> {
        let option_rc = self.children.get(index).map(|x| x.clone());
        option_rc.map(move |x| x.deref_mut_with(self))
    }
    #[inline]
    pub fn children(&self, r: Range<usize>) -> Vec<ForestNodeRc<T>> {
        self.children[r].to_vec()
    }
    #[inline]
    pub fn clone_children(&self) -> Vec<ForestNodeRc<T>> {
        self.children.clone()
    }
    #[inline]
    pub fn iter_children<'a>(&'a self) -> ForestNodeIter<'a, T> {
        ForestNodeIter {
            parent: self,
            cur: 0,
        }
    }
    #[inline]
    pub fn for_each_child<F>(&self, mut f: F) where F: FnMut(&ForestNode<T>) {
        let children = unsafe { &*(&self.children as *const Vec<ForestNodeRc<T>>) };
        for child_rc in children.iter() {
            {
                let child = self.another(child_rc);
                f(child);
            }
        }
    }
    #[inline]
    pub fn for_each_child_mut<F>(&mut self, mut f: F) where F: FnMut(&mut ForestNode<T>) {
        let children = unsafe { &*(&self.children as *const Vec<ForestNodeRc<T>>) };
        for child_rc in children.iter() {
            {
                let child = self.another_mut(child_rc);
                f(child);
            }
        }
    }

    #[inline]
    pub fn rc(&self) -> ForestNodeRc<T> {
        self.self_weak.as_ref().unwrap().upgrade().unwrap()
    }
    #[inline]
    fn replace_from_old_parent(&mut self, new_parent: ForestNodeWeak<T>) {
        let old_parent = self.parent.replace(new_parent);
        match old_parent {
            None => { },
            Some(x) => {
                match x.upgrade() {
                    None => { },
                    Some(parent) => {
                        let self_rc = self.rc();
                        let parent = parent.deref_mut_with(self);
                        let i = parent.find_child_position(&self_rc).unwrap();
                        parent.children.remove(i);
                    }
                }
            }
        }
    }
    pub fn find_child_position(&self, child: &ForestNodeRc<T>) -> Option<usize> {
        self.children.iter().position(|c| {
            ForestNodeRc::ptr_eq(child, &c)
        })
    }
    pub fn append(&mut self, child: ForestNodeRc<T>) {
        let self_rc = self.rc();
        self.children.push(child.clone());
        let c = child.deref_mut_with(self);
        c.replace_from_old_parent(self_rc.downgrade());
        c.content.parent_node_changed();
    }
    pub fn insert(&mut self, child: ForestNodeRc<T>, position: usize) {
        let self_rc = self.rc();
        self.children.insert(position, child.clone());
        let c = child.deref_mut_with(self);
        c.replace_from_old_parent(self_rc.downgrade());
        c.content.parent_node_changed();
    }
    pub fn remove(&mut self, position: usize) -> ForestNodeRc<T> {
        let child = self.children.remove(position);
        let c = child.deref_mut_with(self);
        c.parent.take();
        c.content.parent_node_changed();
        child
    }
    pub fn replace(&mut self, new_child: ForestNodeRc<T>, position: usize) -> ForestNodeRc<T> {
        let self_rc = self.rc();
        let old_child = self.children[position].clone();
        self.children[position] = new_child.clone();
        let c = old_child.deref_mut_with(self);
        c.parent.take();
        c.content.parent_node_changed();
        let c = new_child.deref_mut_with(self);
        c.replace_from_old_parent(self_rc.downgrade());
        c.content.parent_node_changed();
        old_child
    }
    pub fn splice(&mut self, position: usize, removes: usize, inserts: Vec<ForestNodeRc<T>>) -> Box<[ForestNodeRc<T>]> {
        let self_rc = self.rc();
        let removes: Box<[ForestNodeRc<T>]> = self.children.splice(position..(position + removes), inserts.clone()).collect();
        for child in removes.iter() {
            let c = child.deref_mut_with(self);
            c.parent.take();
            c.content.parent_node_changed();
        }
        for child in inserts.iter() {
            let c = child.deref_mut_with(self);
            c.replace_from_old_parent(self_rc.downgrade());
            c.content.parent_node_changed();
        }
        removes
    }

    pub fn iter(&self) -> Iter<ForestNodeRc<T>> {
        self.children.iter()
    }
}

impl<T: ForestNodeContent + Clone> ForestNode<T> {
    pub(crate) fn clone_node(&self) -> Self {
        Self {
            context: self.context.clone(),
            self_weak: None,
            parent: None,
            children: vec![],
            content: self.content.clone(),
        }
    }
}

impl<T: ForestNodeContent> Deref for ForestNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T: ForestNodeContent> DerefMut for ForestNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<'a, T: ForestNodeContent> BitXor<usize> for &'a ForestNode<T> {
    type Output = &'a ForestNode<T>;
    fn bitxor(self, count: usize) -> Self::Output {
        let mut ret = self;
        for _ in 0..count {
            ret = ret.parent().unwrap()
        }
        ret
    }
}

impl<'a, T: ForestNodeContent> BitXor<usize> for &'a mut ForestNode<T> {
    type Output = &'a mut ForestNode<T>;
    fn bitxor(self, count: usize) -> Self::Output {
        let mut ret = self;
        for _ in 0..count {
            ret = ret.parent_mut().unwrap()
        }
        ret
    }
}

impl<T: ForestNodeContent> Index<usize> for ForestNode<T> {
    type Output = ForestNode<T>;
    fn index(&self, index: usize) -> &Self::Output {
        self.child(index).unwrap()
    }
}

impl<T: ForestNodeContent> IndexMut<usize> for ForestNode<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.child_mut(index).unwrap()
    }
}

pub struct ForestNodeIter<'a, T: ForestNodeContent> {
    parent: &'a ForestNode<T>,
    cur: usize,
}

impl<'a, T: ForestNodeContent> Iterator for ForestNodeIter<'a, T> {
    type Item = &'a ForestNode<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.parent.children.len() {
            let cur = self.cur;
            self.cur += 1;
            self.parent.child(cur)
        } else {
            None
        }
    }
}
