use super::node::{HistoryId, HistoryNode, RootHistoryNode};

#[derive(Clone)]
pub struct HistoryGraph {
    nodes: Vec<HistoryNode>,
}

impl HistoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![RootHistoryNode::NoOp.into()],
        }
    }

    pub fn next_id(&mut self) -> HistoryId {
        self.nodes
            .len()
            .try_into()
            .expect("problem with zero length history graph")
    }

    pub fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.next_id();
        self.push(node);
        result
    }
}

impl ::std::ops::Deref for HistoryGraph {
    type Target = Vec<HistoryNode>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl ::std::ops::DerefMut for HistoryGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}
