use crate::{
    config::Config,
    direction::Direction,
    display::Display,
    renderer::{NativeRenderer, Renderer},
    split_direction::SplitDirection,
    system::NativeWindow,
    system::SystemError,
    system::SystemResult,
    system::WindowId,
    tile_grid::{
        text_renderer::TextRenderer, node::Node, node::NodeInfo, 
        graph_wrapper::GraphWrapper, tile_render_info::TileRenderInfo
    },
};
use std::cmp;
use log::{debug, error};

pub mod graph_wrapper;
pub mod node;
pub mod tile_render_info;
pub mod text_renderer;

static FULL_SIZE: u32 = 120;
static HALF_SIZE: u32 = FULL_SIZE / 2;

#[derive(Clone, Debug)]
pub struct TileGrid<TRenderer: Renderer = NativeRenderer> {
    // pub display: Display,
    pub renderer: TRenderer,
    pub id: i32,
    pub taskbar_window: i32,
    pub focused_id: Option<usize>,
    pub fullscreen_id: Option<usize>,
    pub next_axis: SplitDirection,
    pub next_direction: Direction,
    graph: GraphWrapper,
}

impl TileGrid {
    pub fn draw_grid(&self, display: &Display, config: &Config) -> SystemResult {
        debug!("IsFullScreened? {} FocusedNode: {:?}", self.fullscreen_id.is_some(), self.focused_id);
        let render_infos = self.get_render_info(64, 20);
        debug!("{}", TextRenderer::render(64, 20, render_infos)); 

        let (padding, margin) = 
            (if config.inner_gap > 0 { config.inner_gap / 2 } else { 0 }, 
             if config.outer_gap > 0 { config.outer_gap } else { 0 });

        let display_width = display.working_area_width(config) - margin;
        let display_height = display.working_area_height(config) - margin;
        let display_left = display.working_area_left() + (margin / 2);
        let display_top = display.working_area_top(config) + (margin / 2);

        let render_infos = self.get_render_info(display_width as u32, display_height as u32);

        for render_info in render_infos {
            let left_padding = if render_info.x != 0 { padding } else { 0 };
            let top_padding = if render_info.y != 0 { padding } else { 0 };
            let right_padding = 
                if (render_info.x + render_info.width) as i32 != display_width { padding } else { 0 };
            let bottom_padding = 
                if (render_info.y + render_info.height) as i32 != display_height { padding } else { 0 };

            let left = display_left + render_info.x as i32 + left_padding;
            let top = display_top + render_info.y as i32 + top_padding;
            let width = render_info.width as i32 - left_padding - right_padding;
            let height = render_info.height as i32 - top_padding - bottom_padding;

            self.renderer.render(self, &render_info.window, config, display, left, top, width, height)?;
        }

        Ok(())
    }
    /// Returns a list of render information for each tile in the graph
    /// inner/outer padding should be handled outside of the tile grid by reducing the
    /// width/height by the outer padding and trimming off between tiles with the inner padding.
    pub fn get_render_info(&self, width: u32, height: u32) -> Vec::<TileRenderInfo> {
        let mut render_infos = Vec::<TileRenderInfo>::new();

        if let Some(fullscreen_id) = self.fullscreen_id {
            match self.graph.node(fullscreen_id) {
                Node::Tile((node, window)) => {
                    render_infos.push(TileRenderInfo {
                        window: window.clone(),
                        x: 0,
                        y: 0,
                        height: height,
                        width: width,
                        debug_id: fullscreen_id,
                        debug_size: node.size,
                        debug_order: node.order,
                    });
                },
                _ => ()
            }
        }
        else if let Some(root_id) = self.graph.get_root() {
            render_infos = self.populate_render_info(render_infos, root_id, 0, width, 0, height);
        }

        render_infos 
    }
    /// A recursive function that walks the graph and populates the supplied vec with rendering information
    /// for each node based on the given resolution.
    fn populate_render_info(&self, mut render_infos: Vec::<TileRenderInfo>, current_node_id: usize,
                            min_x: u32, max_x: u32, min_y: u32, max_y: u32) -> Vec::<TileRenderInfo> {
        match self.graph.node(current_node_id) {
            Node::Tile((node, window)) => {
                render_infos.push(TileRenderInfo {
                    window: window.clone(),
                    x: min_x,
                    y: min_y,
                    height: if min_y > max_y { 0 } else { max_y - min_y },
                    width: if min_x > max_x { 0 } else { max_x - min_x },
                    debug_id: current_node_id,
                    debug_size: node.size,
                    debug_order: node.order,
                });
            },
            Node::Column(_) => {
                let children = self.graph.get_sorted_children(current_node_id);
                let length = children.len();
                let mut current_min_x = min_x;
                let mut remainder = (max_x - min_x) % children.len() as u32;
                let mut get_remainder_slice = || if remainder > 0 { remainder -= 1; 1 } else { 0 };

                let mut count = 1;
                for child in children {
                    let child_size = self.graph.node(child).get_size();
                    let item_width = (((max_x - min_x) as f32) * 
                                     (child_size as f32 / FULL_SIZE as f32)).floor() as u32;

                    if item_width <= max_x {
                        let remainder_slice = get_remainder_slice();
                        let current_max_x = if count == length { max_x }
                                            else { current_min_x + item_width + remainder_slice };

                        render_infos = self.populate_render_info(render_infos, child, 
                                                                 current_min_x, current_max_x, 
                                                                 min_y, max_y);
                        current_min_x += item_width + remainder_slice; 
                    }

                    count += 1;
                }
            },
            Node::Row(_) => {
                let children = self.graph.get_sorted_children(current_node_id);
                let length = children.len();
                let mut current_min_y = min_y;
                let mut remainder = (max_y - min_y) % children.len() as u32;
                let mut get_remainder_slice = || if remainder > 0 { remainder -= 1; 1 } else { 0 };

                let mut count = 1;
                for child in children {
                    let child_size = self.graph.node(child).get_size();
                    let item_height = (((max_y - min_y) as f32) * 
                                      (child_size as f32 / FULL_SIZE as f32)).floor() as u32;

                    if item_height <= max_y {
                        let remainder_slice = get_remainder_slice();
                        let current_max_y = if count == length { max_y }
                                            else { current_min_y + item_height + remainder_slice };

                        render_infos = self.populate_render_info(render_infos, child, min_x, max_x, 
                                                                 current_min_y, current_max_y);
                        current_min_y += item_height + remainder_slice;
                    }

                    count += 1;
                }
            }
        }

        render_infos 
    }
}

impl<TRenderer: Renderer> TileGrid<TRenderer> {
    pub fn new(id: i32, renderer: TRenderer) -> TileGrid<TRenderer> {
        Self {
            id,
            // display: get_primary_display(),
            renderer,
            taskbar_window: 0,
            graph: GraphWrapper::new(),
            fullscreen_id: None,
            focused_id: None,
            next_axis: SplitDirection::Vertical,
            next_direction: Direction::Right,
        }
    }
    /// Returns whether the tile grid is populated or not
    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }
    /// Iterates and hides every window managed by the current tile grid
    pub fn hide(&self) {
        for node_id in self.graph.nodes() {
            if self.graph.node(node_id).is_tile() {
                self.graph.node(node_id).get_window().hide();
            }
        }
    }
    /// Removes the focused node, if it exists, and returns the window on that node.
    /// Leaves the tile_grid in an unfocused state and un-fullscreens if currently fullscreen'd.
    pub fn pop(&mut self) -> Option<NativeWindow> {
        let removed_node: Option<Node> = self.remove_node(self.focused_id);
        self.focused_id = None;
        self.fullscreen_id = None;

        removed_node.map(|x| x.take_window()) 
    }
    /// Iterates and removes every node while resetting any windows that were managed
    pub fn cleanup(&mut self) -> SystemResult {
        while !self.is_empty() {
            self.focused_id = self.get_last_tile();

            if let Some(mut window) = self.pop() {
                window.cleanup()?;
            }
        }

        Ok(())
    }
    /// Sets the currently focused tile to be fullscreen'd if it's not already, otherwise
    /// reverts the graph to non-fullscreen'd mode. When a tile is fullscreened certain
    /// operations are disabled like managing new tiles changing focus, resizing and tile movement
    pub fn toggle_fullscreen(&mut self) {
        if self.focused_id.is_some() {
            if self.fullscreen_id.is_some() {
                self.fullscreen_id = None;
            } else {
                self.fullscreen_id = self.focused_id;
            }
        }
    }
    /// Travels up the graph from the focused node until it finds a row
    /// and then resets the size of all of that row's children.
    /// No-op if no row is found above the focused node or if a node is currently fullscreen'd.
    pub fn reset_row(&mut self) {
        if self.fullscreen_id.is_some() { return; }
        self.reset_size(self.graph.to_closest_row(self.focused_id))
    }
    /// Gets all the child nodes of a node and re-distrbutes the size among them. 
    /// This applies only one level down, regardless of what type of nodes they are; any
    /// child Row/Column nodes' children will retain their respective size.
    fn reset_size(&mut self, parent_id: Option<usize>) {
        if !parent_id.is_some() || self.fullscreen_id.is_some() { return }

        let children = self.graph.get_sorted_children(parent_id.unwrap());
        let number_of_children = children.len();
        let size_per_child = FULL_SIZE / number_of_children as u32;

        let mut remainder = FULL_SIZE % number_of_children as u32;
        let mut get_remainder_slice = || if remainder > 0 { remainder -= 1; 1 } else { 0 }; 

        for child in children {
            self.graph.node_mut(child)
                      .set_size(size_per_child + get_remainder_slice());
        }
    }
    /// Travels up the graph from the focused node until it finds a column
    /// and then resets the size of all of that column's children.
    /// No-op if no column is found above the focused node or if a node is currently fullscreen'd.
    pub fn reset_column(&mut self) {
        if self.fullscreen_id.is_some() { return; }
        self.reset_size(self.graph.to_closest_column(self.focused_id))
    }
    /// Iterates and shows every window managed by the current tile_grid 
    pub fn show(&self) -> SystemResult {
        for node_id in self.graph.nodes() {
            if self.graph.node(node_id).is_tile() {
                let window = self.graph.node(node_id).get_window();
                window.show();
                window.to_foreground(true)
                      .map_err(SystemError::ShowWindow)?;
                if let Err(e) = window.remove_topmost() {
                    error!("{}", e);
                }
            }
        }

        if let Some(focused_id) = self.focused_id {
            self.graph.node(focused_id).get_window().focus().expect("Failed to focus window");
        }
        Ok(())
    }
    /// Returns the window of the currently focused tile if it exists
    pub fn get_focused_window(&self) -> Option<&NativeWindow> {
        self.focused_id.map(|id| self.graph.node(id).get_window())
    }
    /// Returns the window that matches by ID if it exists
    pub fn get_window(&self, id: WindowId) -> Option<&NativeWindow> {
        self.graph.nodes()
                  .find(|n| {
                      let node = self.graph.node(*n);
                      node.is_tile() && node.get_window().id == id
                  })
                  .map(|n| self.graph.node(n).get_window())
    }
    /// Runs the passed in function on the currently focused tile's window in the current tile grid.
    pub fn modify_focused_window<TFunction>(self: &mut Self, f: TFunction) -> SystemResult
        where 
            TFunction: FnMut(&mut NativeWindow) -> SystemResult + Copy {
        if let Some(focused_id) = self.focused_id {
            self.graph.node_mut(focused_id).modify_window(f)?;
        }
        Ok(())
    }
    /// Iterates across all tile nodes and runs the passed in function on them. Useful for
    /// changing all windows in the current tile grid.
    pub fn modify_windows<TFunction>(self: &mut Self, f: TFunction) -> SystemResult
        where 
            TFunction: FnMut(&mut NativeWindow) -> SystemResult + Copy {
        for node_id in self.graph.nodes() {
            let node = self.graph.node_mut(node_id);
            if node.is_tile() {
                node.modify_window(f)?;
            }
        }
        Ok(())
    }
    pub fn swap_focused(&mut self, direction: Direction) {
        if self.fullscreen_id.is_some() { return; }

        if let Some(parent_id) = self.graph.map_to_parent(self.focused_id) {
            let focused_id = self.focused_id.unwrap();
            let focused_order = self.graph.node(focused_id).get_order();
            let children = self.graph.get_children(parent_id);

            let should_swap_with_sibling = 
                match (&direction, self.graph.node(parent_id)) {
                    (Direction::Left, Node::Column(_)) | 
                    (Direction::Up, Node::Row(_)) => focused_order > 0 && children.len() > 1,
                    (Direction::Right, Node::Column(_)) | 
                    (Direction::Down, Node::Row(_)) => focused_order < (children.len() - 1) as u32,
                    _ => false
                };

            if should_swap_with_sibling {
                let sibling_id = self.graph.get_neighbor(focused_id, direction);
                self.swap_order(focused_id, sibling_id.unwrap());
            } 
        }
    }
    fn swap_order(&mut self, first: usize, second: usize) {
        let first_order = self.graph.node(first).get_order();
        let second_order = self.graph.node(second).get_order();
        
        self.graph.node_mut(first).set_order(second_order);
        self.graph.node_mut(second).set_order(first_order);
    }
    pub fn focus(&mut self, direction: Direction) -> SystemResult {
        if self.fullscreen_id.is_some() || !self.focused_id.is_some() { return Ok(()); }

        let parent_id = self.graph.map_to_parent(self.focused_id);
        if let Some(mut parent_id) = parent_id {
            let mut target_focus: Option<usize> = None;
            let mut current_focus = self.focused_id.unwrap();
            while !target_focus.is_some() {
                let children = self.graph.get_children(parent_id).len();
                let focused_order = self.graph.node(current_focus).get_order();

                let should_focus_sibling = 
                    match (&direction, self.graph.node(parent_id)) {
                        (Direction::Left, Node::Column(_)) | 
                        (Direction::Up, Node::Row(_)) => focused_order > 0 && children > 1,
                        (Direction::Right, Node::Column(_)) | 
                        (Direction::Down, Node::Row(_)) => focused_order < (children - 1) as u32,
                        _ => false
                    };

                if should_focus_sibling {
                    target_focus = self.graph.get_neighbor(current_focus, direction);
                } else if let Some(p_id) = self.graph.map_to_parent(Some(parent_id)) {
                    // focus on parent and iterate again to find a tile in chosen direction
                    current_focus = parent_id;
                    parent_id = p_id;
                } else {
                    // no parent, can't move in direction
                    target_focus = self.focused_id;
                }
            }

            self.focused_id = self.graph.to_closest_tile(target_focus, Some(direction));
            self.graph.node(self.focused_id.unwrap()).get_window().focus()?;
        }

        Ok(())
    }
    fn reset_order(&mut self, parent_id: usize) {
        let nodes = self.graph.get_sorted_children(parent_id);

        let mut order = 0;
        for node in nodes {
            self.graph.node_mut(node).set_order(order);
            order += 1;
        }
    }
    /// Removes and returns the node of the given node_id. The behavior of the removal falls into one of three cases:
    /// Case One: If the graph only has one node and it's the given node, then the graph is emptied. 
    /// Case Two: If the given node has only one sibling then the node is removed and its sibling gets propogated 
    /// "up" a level to take the place of its parent node (A parent node with only one child is an invalid state). 
    /// Case Three: If the given node has more than one sibling then the node is removed and its size is distributed among its siblings
    fn remove_node(&mut self, node_id: Option<usize>) -> Option<Node> {
        let mut removed_node: Option<Node> = None;
        if let Some(current_id) = node_id {
            if let Some(parent_id) = self.graph.map_to_parent(Some(current_id)) {
                let children = self.graph.get_children(parent_id);
                let number_of_children = children.len();
                if number_of_children == 2 {
                    // remove the current item
                    // make the other child take place of parent
                    let sibling_id = *children.iter().find(|x| **x != current_id).unwrap();
                    
                    if let Some(grand_parent_id) = self.graph.map_to_parent(Some(parent_id)) {
                        self.graph.connect(grand_parent_id, sibling_id);
                    }

                    let (order, size) = self.graph.node(parent_id).get_info();

                    let keep_node = self.graph.node_mut(sibling_id);
                    keep_node.set_info(order, size);

                    self.graph.remove_node(parent_id);
                    removed_node = self.graph.remove_node(current_id);
                } else {
                    // remove the current item
                    // distribute size among siblings
                    let size = self.graph.node(current_id).get_size();
                    let size_per_sibling = size / (number_of_children - 1) as u32;

                    let mut remainder = size % (number_of_children - 1) as u32;
                    let mut get_remainder_slice = || if remainder > 0 { remainder -= 1; 1 } else { 0 }; 


                    for child in children {
                        if child != current_id {
                            let current_size = self.graph.node(child).get_size();

                            self.graph.node_mut(child)
                                      .set_size(size_per_sibling + current_size + get_remainder_slice());
                        }
                    }
                    
                    removed_node = self.graph.remove_node(current_id);
                    self.reset_order(parent_id);
                }
            } else { 
                // focused is root node so empy out entire graph
                removed_node = self.graph.remove_node(current_id);
                self.graph.clear();
            }
        } 

        removed_node
    }
    pub fn close_focused(&mut self) -> Option<NativeWindow> {
        if let Some(focused_node) = self.focused_id.map(|id| self.graph.node(id)) {
            self.remove_by_window_id(focused_node.get_window().id);
        }

        None
    }
    pub fn remove_by_window_id(&mut self, id: WindowId) -> Option<NativeWindow> {
        let mut window: Option<NativeWindow> = None;
        if let Some(node_id) = self.graph.find(|x| x.is_tile() && x.get_window().id == id) {
            window = self.remove_node(Some(node_id)).map(|x| x.take_window());
            if let Some(focused_id) = self.focused_id {
                if focused_id == node_id {
                    self.focused_id = None;
                }
            }
            if let Some(fullscreen_id) = self.fullscreen_id {
                if fullscreen_id == node_id || self.graph.nodes().count() <= 1 {
                    self.fullscreen_id = None;
                }
            }
        }

        window
    }
    /// Returns whether a given window ID exists in the tile grid 
    pub fn contains(&self, window_id: WindowId) -> bool {
        self.graph.nodes()
                  .find(|n| {
                      let node = self.graph.node(*n);
                      node.is_tile() && node.get_window().id == window_id
                  })
                  .is_some()  
    }
    /// Sets the currently focused tile to whatever happens to be "last" in the graph.
    /// See get_last_tile for more information.
    pub fn focus_last_tile(self: &mut Self) {
        self.focused_id = self.get_last_tile();
    }
    /// Returns the an Option NodeID (usize) of the last Tile in the tile grid.
    /// This is somewhat arbitrary as it won't necessarily be the last node added to
    /// the grid based on the graph implementation but can serve as a "give me a node toward the 'bottom'
    /// of the graph."
    fn get_last_tile(&self) -> Option<usize> {
        self.graph.nodes().filter(|n| self.graph.node(*n).is_tile()).last()
    }
    /// Focuses the tile that holds the given window ID if it exists in the current tile grid
    pub fn focus_tile_by_window_id(&mut self, window_id: WindowId) {
        let maybe_window_tile = self.graph.nodes()
                                    .find(|n| {
                                        let node = self.graph.node(*n);
                                        node.is_tile() && node.get_window().id == window_id
                                    });
        if maybe_window_tile.is_some() {
            self.focused_id = maybe_window_tile;
        }
    }
    pub fn push(&mut self, window: NativeWindow) {
        if self.fullscreen_id.is_some() {
            // don't do anything if fullscreened
            return;
        }

        if self.graph.len() == 0 {
            let new_root_node = Node::Tile((NodeInfo { order: 0, size: FULL_SIZE }, window));
            self.focused_id = Some(self.graph.add_node(new_root_node));

            // first node inserted in empty graph so return early
            return;
        }

        if self.contains(window.id) {
            // window is already in graph
            return;
        }

        if !self.focused_id.is_some() {
            // if we're not focused, just focus last tile in the graph
            self.focused_id = self.get_last_tile();
        }

        if let Some(current_id) = self.focused_id {
            let mut new_node = Node::Tile((NodeInfo { order: 0, size: 0 }, window));
            let (existing_node_order, new_node_order) = match self.next_direction {
                                                            Direction::Up | Direction::Left => (1, 0),
                                                            _ => (0, 1)
                                                        };
            match self.graph.node(current_id) {
                Node::Tile(_) => {
                    if let Some(parent_id) = self.graph.map_to_parent(Some(current_id)) {
                        type CreateNode = fn(u32, u32) -> Node;
                        enum PushOperation {
                            AppendToParent,
                            SwapAndAppend(CreateNode)
                        }

                        let operation = match (self.graph.node(parent_id), self.next_axis) {
                            (Node::Column(_), SplitDirection::Vertical) |
                            (Node::Row(_), SplitDirection::Horizontal) => PushOperation::AppendToParent,
                            (Node::Column(_), _) => PushOperation::SwapAndAppend(Node::row),
                            (Node::Row(_), _) => PushOperation::SwapAndAppend(Node::column),
                            _ => panic!("Parent not column or row")
                        };

                        match operation {
                            PushOperation::AppendToParent => {
                                // parent is same type as what we want to add
                                // so append item to parent's column list
                                let (current_node_order, ..) = self.graph.node(current_id).get_info();
                                let new_node_order = current_node_order + new_node_order;
                                new_node.set_info(new_node_order, self.make_space_for_node(parent_id));
                                self.shift_order(parent_id, new_node_order);
                                self.focused_id = Some(self.graph.add_child(parent_id, new_node));
                            },
                            PushOperation::SwapAndAppend(create_node) => {
                                // parent is opposite type of what we want to add
                                // so swap current node with opposite of parent type node 
                                // and append current item + new item there
                                let (new_order, new_size) = self.graph.node(current_id).get_info();
                                let new_parent_node = create_node(new_order, new_size);

                                let (new_parent_id, child_id) = self.graph.swap_and_nest(current_id, new_parent_node);
                                self.graph.node_mut(child_id).set_info(existing_node_order, HALF_SIZE); 
                                new_node.set_info(new_node_order, HALF_SIZE);
                                self.focused_id = Some(self.graph.add_child(new_parent_id, new_node));
                            }
                        }
                    } else /* must be root tile */ { 
                        let new_parent = match self.next_axis {
                                           SplitDirection::Vertical => Node::column(0, FULL_SIZE),
                                           SplitDirection::Horizontal => Node::row(0, FULL_SIZE)
                                       };
                        
                        let (new_parent_id, child_id) = self.graph.swap_and_nest(current_id, new_parent);
                        self.graph.node_mut(child_id).set_info(existing_node_order, HALF_SIZE); 
                        new_node.set_info(new_node_order, HALF_SIZE);
                        self.focused_id = Some(self.graph.add_child(new_parent_id, new_node));
                    }
                }
                _ => panic!("Focused node not a tile. This is an invalid state")
            }
        }
    }
    fn shift_order(&mut self, parent_id: usize, mut shift_point: u32) {
        let nodes = self.graph.get_sorted_children(parent_id);
        let nodes = nodes.iter()
                         .filter(|x| self.graph.node(**x).get_info().0 >= shift_point)
                         .collect::<Vec<_>>();

        for node in nodes {
            shift_point += 1;
            self.graph.node_mut(*node).set_order(shift_point);
        }
    }
    fn make_space_for_node(&mut self, parent_id: usize) -> u32 {
        let mut children = self.graph.get_children(parent_id);
        let target_size_of_new_item = (FULL_SIZE as f32 / (children.len() as f32 + 1.0)).floor();
        let mut existing_children_total = 0;

        let take_from_each = (target_size_of_new_item / children.len() as f32) as u32;
        let mut remainder = (target_size_of_new_item % children.len() as f32) as u32;
        let mut take_remainder = || if remainder > 0 { remainder -= 1; 1 } else { 0 };

        children.sort_by_key(|x| self.graph.node(*x).get_size());
        children.reverse();

        for child_id in children {
            let mut child_size = self.graph.node(child_id).get_size();
            child_size -= take_from_each + take_remainder(); 

            existing_children_total += child_size;
            self.graph.node_mut(child_id).set_size(child_size);
        }
        
        FULL_SIZE - existing_children_total 
    }
    pub fn trade_size_with_neighbor(&mut self, node_id: Option<usize>, direction: Direction, size: i32) {
        if !node_id.is_some() || self.fullscreen_id.is_some() { return; }

        if let Some(parent_id) = self.graph.map_to_parent(node_id) {
            let node_id = node_id.unwrap();
            let (node_order, node_size) = self.graph.node(node_id).get_info();
            let children = self.graph.get_children(parent_id);

            match (direction, &self.graph.node(parent_id)) {
                (Direction::Left, Node::Column(_)) | 
                (Direction::Up, Node::Row(_)) if node_order > 0 => {
                    if let Some(neighbor_id) = children.iter()
                                                       .find(|x| self.graph.node(**x).get_order() == node_order - 1) {
                        let neighbor_size = self.graph.node(*neighbor_id).get_size();

                        if size > 0 && neighbor_size > size.abs() as u32 
                            || size < 0 && node_size > size.abs() as u32 {
                            self.graph.node_mut(*neighbor_id).set_size((neighbor_size as i32 - size) as u32);
                            self.graph.node_mut(node_id).set_size((node_size as i32 + size) as u32);
                        }
                    }
                }
                (Direction::Right, Node::Column(_)) | 
                (Direction::Down, Node::Row(_)) => {
                    if let Some(neighbor_id) = children.iter()
                                                       .find(|x| self.graph.node(**x).get_order() == node_order + 1) {
                        let neighbor_size = self.graph.node(*neighbor_id).get_size();

                        if size > 0 && neighbor_size > size.abs() as u32 
                            || size < 0 && node_size > size.abs() as u32 {
                            self.graph.node_mut(*neighbor_id).set_size((neighbor_size as i32 - size) as u32);
                            self.graph.node_mut(node_id).set_size((node_size as i32 + size) as u32);
                        }
                    }
                }
                _ => self.trade_size_with_neighbor(Some(parent_id), direction, size)
            }
        }
    }
    pub fn move_focused_out(&mut self, direction: Direction) {
        if self.fullscreen_id.is_some() { return; }

        if let Some(parent_id) = self.graph.map_to_parent(self.focused_id) {
            let focused_id = self.focused_id.unwrap();
            let children = self.graph.get_children(parent_id);

            if !self.graph.map_to_parent(Some(parent_id)).is_some() {
                let new_root = match self.graph.node(parent_id) {
                                  Node::Column(_) => Node::row(0, FULL_SIZE),
                                  Node::Row(_) => Node::column(0, FULL_SIZE),
                                  _ => panic!("Parent must be row or column")
                               };
                if children.len() == 2 && self.graph.node(children[0]).is_tile() 
                                       && self.graph.node(children[1]).is_tile() {
                    self.graph.swap_node(parent_id, new_root);
                } else if children.len() > 2 {
                    self.remove_child(parent_id, focused_id);
                    let new_root_id = self.graph.add_node(new_root);
                    self.graph.connect(new_root_id, parent_id);
                    self.graph.connect(new_root_id, focused_id);

                    let (left, right) = match direction {
                                            Direction::Left | Direction::Up => (focused_id, parent_id),
                                            Direction::Right | Direction::Down => (parent_id, focused_id)
                                        };
                    self.graph.node_mut(left).set_info(0, HALF_SIZE);
                    self.graph.node_mut(right).set_info(1, HALF_SIZE);
                }

                return;
            }

            let (new_parent_id, sibling_id) = 
                match (&direction, self.graph.node(parent_id)) {
                    (Direction::Left, Node::Column(_)) | 
                    (Direction::Up, Node::Row(_)) |
                    (Direction::Right, Node::Column(_)) | 
                    (Direction::Down, Node::Row(_)) => 
                        (self.graph.map_to_parent(self.graph.map_to_parent(Some(parent_id))),
                         self.graph.map_to_parent(Some(parent_id))),
                    (Direction::Up, Node::Column(_)) | 
                    (Direction::Left, Node::Row(_)) |
                    (Direction::Down, Node::Column(_)) | 
                    (Direction::Right, Node::Row(_)) => 
                        (self.graph.map_to_parent(Some(parent_id)), Some(parent_id)),
                    _ => (None, None)
                };

            if let Some(new_parent_id) = new_parent_id {
                let sibling_id = sibling_id.unwrap();
                let sibling_order = self.graph.node(sibling_id).get_order();
                let (sibling_order, new_focused_order) = 
                            match &direction {
                                Direction::Up | Direction::Left => (sibling_order + 1, sibling_order),
                                _ => (sibling_order, sibling_order + 1)
                            };

                match &self.graph.node(new_parent_id) {
                    Node::Column(_) | Node::Row(_) => { 
                        self.remove_child(parent_id, focused_id);
                        let new_size = self.make_space_for_node(new_parent_id);
                        self.graph.node_mut(focused_id).set_info(new_focused_order, new_size);
                        self.graph.node_mut(sibling_id).set_order(sibling_order);
                        self.graph.connect(new_parent_id, focused_id);
                        self.reset_order(new_parent_id);
                    },
                    _ => panic!("Expected Column/Row. Tile is not a valid state")
                }

                self.bubble_siblingless_child(parent_id);
            }
        }
    }
    pub fn move_focused_in(&mut self, direction: Direction) {
        if self.fullscreen_id.is_some() { return; }

        if let Some(parent_id) = self.graph.map_to_parent(self.focused_id) {
            let focused_id = self.focused_id.unwrap();
            let number_of_children = self.graph.get_children(parent_id).len();

            if let Some(sibling_id) = self.graph.get_neighbor(focused_id, direction) {
                match &self.graph.node(sibling_id) {
                    Node::Column(_) | Node::Row(_) => { // move focused under sibling column/row
                        self.remove_child(parent_id, focused_id);
                        let new_size = self.make_space_for_node(sibling_id);
                        let new_order = self.graph.get_children(sibling_id).len() as u32;
                        self.graph.node_mut(focused_id).set_info(new_order, new_size);
                        self.graph.connect(sibling_id, focused_id);
                    },
                    Node::Tile(_) => {
                        if number_of_children == 2 {
                            // don't do anything if there are only two nodes and they're both tiles
                            // this prevents columns in columns or rows in rows
                            // in this scenario, the user should move_out not move_in
                            return;
                        }
                        // need to look at what the grandparent is to determine if this
                        // should make a column or row and then nest the sibling tile
                        // and append the current item to it
                        if let Some(sibling_parent_id) = self.graph.map_to_parent(Some(sibling_id)) {
                            let focused_order = self.graph.node(focused_id).get_order();
                            let (sibling_order, sibling_size) = self.graph.node(sibling_id).get_info();
                            let new_order = cmp::min(focused_order, sibling_order);
                            let new_node = match &self.graph.node(sibling_parent_id) {
                                               Node::Column(_) => Node::row(new_order, sibling_size),
                                               Node::Row(_) => Node::column(new_order, sibling_size),
                                               _ => panic!("Parent should be a row or column")
                                           };

                            let new_node_id = self.graph.add_node(new_node);
                            self.graph.disconnect(sibling_parent_id, sibling_id);
                            self.graph.connect(sibling_parent_id, new_node_id);
                            self.remove_child(parent_id, focused_id);

                            self.graph.node_mut(sibling_id).set_info(0, HALF_SIZE);
                            self.graph.node_mut(focused_id).set_info(1, HALF_SIZE);

                            self.graph.connect(new_node_id, sibling_id);
                            self.graph.connect(new_node_id, focused_id);
                        } else {
                            panic!("Not sure if this is a valid scenario");
                        }
                    }
                }

                self.bubble_siblingless_child(parent_id);
            }
        }
    }
    /// Scenario: moving out of a column/row leaving one child behind. This function
    /// swaps the column/row with the remaining child and deletes the column/row node
    fn bubble_siblingless_child(&mut self, parent_id: usize) {
        let children = self.graph.get_children(parent_id);
        if children.iter().len() == 1 {
            let child_id = children[0];
            
            if let Some(grand_parent_id) = self.graph.map_to_parent(Some(parent_id)) {
                self.graph.connect(grand_parent_id, child_id);
            }

            let (order, size) = self.graph.node(parent_id).get_info();
            self.graph.node_mut(child_id).set_info(order, size);

            self.graph.remove_node(parent_id);
        }
    }
    fn remove_child(&mut self, parent_id: usize, child_id: usize) {
        let children = self.graph.get_children(parent_id);
        let number_of_children = children.iter().len();
        let size = self.graph.node(child_id).get_size();
        let size_per_sibling = size / number_of_children as u32;

        let mut remainder = size % number_of_children as u32;
        let mut get_remainder_slice = || if remainder > 0 { remainder -= 1; 1 } else { 0 }; 

        for child in children {
            if child != child_id {
                let child_size = self.graph.node(child).get_size();
                self.graph.node_mut(child).set_size(size_per_sibling + child_size + get_remainder_slice());
            }
        }

        self.graph.disconnect(parent_id, child_id);
        self.reset_order(parent_id);
    }
}

#[cfg(test)]
mod tests;
