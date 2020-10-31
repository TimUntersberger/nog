use crate::system::{NativeWindow, SystemResult};
use log::error;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub order: u32,
    pub size: u32 
}

#[derive(Debug, Clone)]
pub enum Node {
    Column(NodeInfo),
    Row(NodeInfo),
    Tile((NodeInfo, NativeWindow)),
}

impl Node {
    pub fn row(order: u32, size: u32) -> Node {
        Node::Row(NodeInfo { order, size })
    }

    pub fn column(order: u32, size: u32) -> Node {
        Node::Column(NodeInfo { order, size })
    }

    pub fn is_tile(self: &Self) -> bool {
        match self {
            Node::Tile(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_column(self: &Self) -> bool {
        match self {
            Node::Column(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_row(self: &Self) -> bool {
        match self {
            Node::Row(_) => true,
            _ => false
        }
    }

    pub fn set_info(self: &mut Self, order: u32, size: u32) {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => {
                n.order = order;
                n.size = size;
            }
        }
    }

    pub fn get_info(self: &Self) -> (u32, u32) {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => (n.order, n.size)
        }
    }

    pub fn set_size(self: &mut Self, size: u32) {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => n.size = size
        }
    }

    pub fn set_order(self: &mut Self, order: u32) {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => n.order = order
        }
    }

    pub fn get_size(self: &Self) -> u32 {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => n.size
        }
    }

    pub fn get_order(self: &Self) -> u32 {
        match self {
            Node::Column(n) | Node::Row(n) | Node::Tile((n, _)) => n.order
        }
    }

    pub fn get_window(self: &Self) -> &NativeWindow {
        match self {
            Node::Tile((_, w)) => &w,
            _ => panic!("Attempt to get window of non-Tile node")
        }
    }

    pub fn get_window_mut(self: &mut Self) -> &mut NativeWindow {
        match self {
            Node::Tile((_, w)) => w,
            _ => panic!("Attempt to get window of non-Tile node")
        }
    }

    pub fn modify_window<TFunction>(self: &mut Self, mut f: TFunction) -> SystemResult
    where
        TFunction: FnMut(&mut NativeWindow) -> SystemResult {
        match self {
            Node::Tile((_, w)) => f(w),
            _ => {
                error!("Attempt to modify window of non-Tile node");
                Ok(())
            }
        }
    }

    pub fn take_window(self: Self) -> NativeWindow {
        match self {
            Node::Tile((_, w)) => w,
            _ => panic!("Attempt to take window of non-Tile node")
        }
    }
}
