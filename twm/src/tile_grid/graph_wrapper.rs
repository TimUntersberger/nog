use crate::direction::Direction;
use crate::tile_grid::node::Node;
use petgraph::{
    graph::NodeIndex, stable_graph::StableGraph, visit::EdgeRef, Direction as GraphDirection,
};
use std::{fmt, mem};

static EDGE: u32 = 0;

pub struct GraphWrapper {
    graph: StableGraph<Node, u32>,
}

impl Clone for GraphWrapper {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
        }
    }
}

impl fmt::Debug for GraphWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphWrapper")
            .field("Length", &self.len())
            .finish()
    }
}

impl GraphWrapper {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::<Node, u32>::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> usize {
        self.graph.add_node(node).index()
    }

    pub fn remove_node(&mut self, node_id: usize) -> Option<Node> {
        self.graph.remove_node(NodeIndex::new(node_id))
    }

    pub fn clear(&mut self) {
        self.graph.clear();
    }

    pub fn swap_node(&mut self, node_id: usize, mut node: Node) -> Node {
        mem::swap(self.node_mut(node_id), &mut node);
        node
    }

    pub fn add_child(&mut self, parent_id: usize, node: Node) -> usize {
        let child_id = self.add_node(node);
        self.connect(parent_id, child_id);
        child_id
    }

    pub fn swap_and_nest(&mut self, node_id: usize, mut swap_item: Node) -> (usize, usize) {
        mem::swap(self.node_mut(node_id), &mut swap_item);
        (node_id, self.add_child(node_id, swap_item))
    }

    pub fn connect(&mut self, parent_id: usize, child_id: usize) {
        self.graph
            .update_edge(NodeIndex::new(parent_id), NodeIndex::new(child_id), EDGE);
    }

    pub fn disconnect(&mut self, parent_id: usize, child_id: usize) {
        if let Some(edge) = self
            .graph
            .find_edge(NodeIndex::new(parent_id), NodeIndex::new(child_id))
        {
            self.graph.remove_edge(edge);
        }
    }

    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    pub fn node(&self, id: usize) -> &Node {
        &self.graph[NodeIndex::new(id)]
    }

    pub fn node_mut(&mut self, id: usize) -> &mut Node {
        &mut self.graph[NodeIndex::new(id)]
    }

    pub fn map_to_parent(&self, id: Option<usize>) -> Option<usize> {
        id.and_then(|i| {
            self.graph
                .neighbors_directed(NodeIndex::new(i), GraphDirection::Incoming)
                .next()
        })
        .map(|n| n.index())
    }

    pub fn get_root(&self) -> Option<usize> {
        self.graph.node_indices().find_map(|x| {
            if !self.map_to_parent(Some(x.index())).is_some() {
                Some(x.index())
            } else {
                None
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    pub fn nodes(&self) -> impl Iterator<Item = usize> {
        self.graph
            .node_indices()
            .map(|n| n.index())
            .collect::<Vec<usize>>()
            .into_iter()
    }

    pub fn find<F>(&self, mut f: F) -> Option<usize>
    where
        F: FnMut(&Node) -> bool,
    {
        self.graph.node_indices().find_map(|x| {
            if f(self.node(x.index())) {
                Some(x.index())
            } else {
                None
            }
        })
    }

    pub fn get_children(&self, parent_id: usize) -> Vec<usize> {
        self.graph
            .edges_directed(NodeIndex::new(parent_id), GraphDirection::Outgoing)
            .map(|x| self.graph.edge_endpoints(x.id()).unwrap())
            .map(|(_, child)| child.index())
            .collect()
    }

    pub fn get_sorted_children(&self, parent_id: usize) -> Vec<usize> {
        let mut children = self.get_children(parent_id);
        children.sort_by_key(|x| self.node(*x).get_info().0);

        children
    }

    pub fn get_neighbor(&self, id: usize, dir: Direction) -> Option<usize> {
        let order = self.node(id).get_order();
        if let Some(parent_id) = self.map_to_parent(Some(id)) {
            let neighbors = self.get_children(parent_id);

            match (dir, &self.node(parent_id)) {
                (Direction::Left, Node::Column(_)) | (Direction::Up, Node::Row(_)) if order > 0 => {
                    neighbors.iter().find_map(|x| {
                        if self.node(*x).get_order() == order - 1 {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                }
                (Direction::Right, Node::Column(_)) | (Direction::Down, Node::Row(_)) => {
                    neighbors.iter().find_map(|x| {
                        if self.node(*x).get_order() == order + 1 {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn to_closest_row(&self, id: Option<usize>) -> Option<usize> {
        if let Some(parent_id) = self.map_to_parent(id) {
            match &self.node(parent_id) {
                Node::Row(_) => Some(parent_id),
                _ => self.to_closest_row(Some(parent_id)),
            }
        } else {
            None
        }
    }

    pub fn to_closest_column(&self, id: Option<usize>) -> Option<usize> {
        if let Some(parent_id) = self.map_to_parent(id) {
            match &self.node(parent_id) {
                Node::Column(_) => Some(parent_id),
                _ => self.to_closest_column(Some(parent_id)),
            }
        } else {
            None
        }
    }

    pub fn to_closest_tile(
        &self,
        id: Option<usize>,
        moving_direction: Option<Direction>,
    ) -> Option<usize> {
        if let Some(id) = id {
            match &self.node(id) {
                Node::Column(_) | Node::Row(_) => {
                    let mut children = self.get_sorted_children(id);
                    self.to_closest_tile(
                        if children.len() > 0 {
                            match (&self.node(id), moving_direction) {
                                (Node::Column(_), Some(Direction::Left))
                                | (Node::Row(_), Some(Direction::Up)) => children.pop(),
                                _ => Some(children[0]),
                            }
                        } else {
                            Some(id)
                        },
                        moving_direction,
                    )
                }
                _ => Some(id),
            }
        } else {
            None
        }
    }
}
