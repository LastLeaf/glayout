use std::rc::Rc;
use super::*;

pub struct ForestNode<T: ForestNodeContent> {
    status: Rc<ForestStatus>,
    content: T,
}

impl<T: ForestNodeContent> ForestNode<T> {
    pub(crate) fn new(forest: &mut Forest<T>) -> Self {
        let ret = Self {
            status: forest.status.clone(),
            content: T::create(forest),
        };
        ret
    }
}

impl<T: ForestNodeContent + Clone> ForestNode<T> {
    pub fn clone_node() -> Self {
        unimplemented!();
    }
}
