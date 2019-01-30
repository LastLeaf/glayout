use std::rc::Rc;
use std::marker::PhantomData;

mod forest_node;
pub use self::forest_node::ForestNode;
mod forest_node_content;
pub use self::forest_node_content::ForestNodeContent;
mod forest_node_rc;
pub use self::forest_node_rc::{ForestNodeRc, ForestNodeWeak, ForestNodeRef, ForestNodeRefMut, ForestNodePtr};
mod forest_node_self;
pub use self::forest_node_self::ForestNodeSelf;
mod forest_context;
use self::forest_context::{ForestContext, ForestContextRef, ForestContextRefMut};

pub struct Forest<T: ForestNodeContent> {
    context: Rc<ForestContext>,
    phantom_data: PhantomData<T>,
}

impl<T: ForestNodeContent> Forest<T> {
    pub fn new() -> Self {
        Self {
            context: Rc::new(ForestContext::new()),
            phantom_data: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
