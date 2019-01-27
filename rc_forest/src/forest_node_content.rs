use super::*;

pub trait ForestNodeContent {
    #[inline]
    fn create(forest: &mut Forest<Self>) -> Self where Self: Sized;
    #[inline]
    fn associate_node(&self, _node: ForestNodeWeak<Self>) where Self: Sized { }
    #[inline]
    fn parent_node_changed(&self, _parent_node: Option<ForestNodeRc<Self>>) where Self: Sized { }
}
