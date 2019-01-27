use std::rc::Rc;
use std::marker::PhantomData;

mod forest_node;
pub use self::forest_node::ForestNode;
mod forest_node_content;
pub use self::forest_node_content::ForestNodeContent;
mod forest_node_rc;
pub use self::forest_node_rc::{ForestNodeRc, ForestNodeWeak, ForestNodeRef, ForestNodeRefMut};
mod forest_status;
use self::forest_status::{ForestStatus, ForestStatusRef, ForestStatusRefMut};

pub struct Forest<T: ForestNodeContent> {
    status: Rc<ForestStatus>,
    phantom_data: PhantomData<T>,
}

impl<T: ForestNodeContent> Forest<T> {
    pub fn new() -> Self {
        Self {
            status: Rc::new(ForestStatus::new()),
            phantom_data: PhantomData,
        }
    }
    pub fn create_node(&mut self) -> ForestNodeRc<T> {
        ForestNodeRc::new(self, ForestNode::new(self))
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
