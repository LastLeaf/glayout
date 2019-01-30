use super::*;

pub trait ForestNodeContent {
    #[inline]
    fn associate_node(&mut self, _node: ForestNodeSelf<Self>) where Self: Sized { }
    #[inline]
    fn parent_node_changed(&mut self) where Self: Sized { }
}
