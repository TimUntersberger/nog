use super::TileGrid;
use super::text_renderer::TextRenderer;
use super::node::{ Node, NodeInfo };
use crate::{ direction::Direction, split_direction::SplitDirection };
use crate::window::Window;
use crate::display::Display;
use std::sync::Mutex;
use winapi::shared::windef::{HWND, HMONITOR, RECT};
use lazy_static::lazy_static;
use crate::{
    config::Config, system::SystemResult, system::NativeWindow, renderer::Renderer, system::WindowId, 
};

fn create_window(id: i32) -> NativeWindow {
    let mut window = NativeWindow::new();
    window.id = WindowId::from(id);
    window
}

fn get_window_id(tile_grid: &TileGrid<TestRenderer>, node_id: usize) -> i32 {
    match tile_grid.graph.node(node_id) {
        Node::Tile((_, w)) => w.id.into(),
        _ => panic!("Expected tile")
    }
}

fn is_column(tile_grid: &TileGrid<TestRenderer>, node_id: usize) -> bool {
    match tile_grid.graph.node(node_id) {
        Node::Column(_) => true,
        _ => false
    }
}

fn is_row(tile_grid: &TileGrid<TestRenderer>, node_id: usize) -> bool {
    match tile_grid.graph.node(node_id) {
        Node::Row(_) => true,
        _ => false
    }
}

fn is_tile(tile_grid: &TileGrid<TestRenderer>, node_id: usize) -> bool {
    match tile_grid.graph.node(node_id) {
        Node::Tile((_,_)) => true,
        _ => false
    }
}

fn perform_actions(tile_grid: &mut TileGrid<TestRenderer>, actions: &str) {
    let mut window_id = 0;
    let mut window_generator = || { 
        window_id += 1; 
        create_window(window_id)
    };

    for action in actions.split(",") {
        match action {
            "p" => tile_grid.push(window_generator()),
            "o" => { tile_grid.pop(); },
            "full" => tile_grid.toggle_fullscreen(),
            "rc" => tile_grid.reset_column(),
            "rr" => tile_grid.reset_row(),
            "sl" => tile_grid.swap_focused(Direction::Left),
            "sd" => tile_grid.swap_focused(Direction::Down),
            "su" => tile_grid.swap_focused(Direction::Up),
            "sr" => tile_grid.swap_focused(Direction::Right),
            "fl" => { tile_grid.focus(Direction::Left); },
            "fd" => { tile_grid.focus(Direction::Down); },
            "fu" => { tile_grid.focus(Direction::Up); },
            "fr" => { tile_grid.focus(Direction::Right); },
            "mil" => { tile_grid.move_focused_in(Direction::Left); },
            "mid" => { tile_grid.move_focused_in(Direction::Down); },
            "miu" => { tile_grid.move_focused_in(Direction::Up); },
            "mir" => { tile_grid.move_focused_in(Direction::Right); },
            "mol" => { tile_grid.move_focused_out(Direction::Left); },
            "mod" => { tile_grid.move_focused_out(Direction::Down); },
            "mou" => { tile_grid.move_focused_out(Direction::Up); },
            "mor" => { tile_grid.move_focused_out(Direction::Right); },
            "axh" => tile_grid.next_axis = SplitDirection::Horizontal,
            "axv" => tile_grid.next_axis = SplitDirection::Vertical,
            "dirl" => tile_grid.next_direction = Direction::Left,
            "dird" => tile_grid.next_direction = Direction::Down,
            "diru" => tile_grid.next_direction = Direction::Up,
            "dirr" => tile_grid.next_direction = Direction::Right,
            _ => ()
        }
    }
}

/* Target:
                             Note: the 0-2-1 sequence here is 10, 12, 11
                                                         v
         c                   [1][1][1][1][1][1][2][2][2][2][2][2]
        / \                  [1][1][1][1][1][1][2][2][2][2][2][2]
       1   r_______          [1][1][1][1][1][1][2][2][2][2][2][2]
           | | |  |\         [1][1][1][1][1][1][2][2][2][2][2][2]
           2 3 c  5 4        [1][1][1][1][1][1][2][2][2][2][2][2]
           ____|_____        [1][1][1][1][1][1][2][2][2][2][2][2]
           |  | | | |        [1][1][1][1][1][1][3][3][3][3][3][3]
           6  7 r 9 8        [1][1][1][1][1][1][6][6][7][0][9][8]
               /|\           [1][1][1][1][1][1][6][6][7][2][9][8]
              / | \          [1][1][1][1][1][1][6][6][7][1][9][8]
             10 12 11        [1][1][1][1][1][1][5][5][5][5][5][5]
                             [1][1][1][1][1][1][4][4][4][4][4][4]
*/
const LARGE_LAYOUT: &str = "p,p,axh,dird,p,p,diru,p,p,axv,dirr,p,p,dirl,p,p,axh,dird,p,diru,p";

#[test]
fn push_node_on_empty_graph() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );

    let window = create_window(123);
    assert_eq!(0, tile_grid.graph.len());
    assert!(!tile_grid.focused_id.is_some());

    tile_grid.push(window);

    assert_eq!(1, tile_grid.graph.len());
    assert_eq!(0, tile_grid.focused_id.unwrap());
}

#[test]
fn push_node_on_populated_root() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let root_node_value = 123;
    let new_node_value = 456;

    let root_window =create_window(root_node_value);
    let new_node_window =create_window(new_node_value);

    tile_grid.push(root_window);
    tile_grid.push(new_node_window);

    assert_eq!(3, tile_grid.graph.len(), "Expected 3 nodes: two added and one parent");
    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_column(&tile_grid, 0), "Expected root node to be column");
    assert_eq!(2, tile_grid.focused_id.unwrap(), "Focused node should be the second node added");
    assert_eq!(root_node_value, 
               get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]),
               "Expected first window added to be left node in graph");
    assert_eq!(new_node_value, 
               get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[1]),
               "Expected second window added to be right node in graph");
    assert_eq!(tile_grid.graph.map_to_parent(tile_grid.focused_id).unwrap(), 
               tile_grid.graph.get_root().unwrap(), "Expected focused item to be child of root");
}

#[test]
fn push_six_column_nodes() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.push(window_generator()); //  push [1]
    tile_grid.push(window_generator()); //  push [1][2]
    tile_grid.push(window_generator()); //  push [1][2][3]
    tile_grid.push(window_generator()); //  push [1][2][3][4]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]

    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_column(&tile_grid, 0), "Expected root node to be column");
    assert_eq!(7, tile_grid.graph.len(), "Expected 7 nodes: 6 added and 1 column parent");
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]));
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[1]));
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[2]));
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[3]));
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[4]));
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[5]));
}

#[test]
fn push_six_column_nodes_altering_direction() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.push(window_generator()); //  push [1]
    tile_grid.push(window_generator()); //  push [1][2]
    tile_grid.next_direction = Direction::Left;
    tile_grid.push(window_generator()); //  push [1][3][2]
    tile_grid.push(window_generator()); //  push [1][4][3][2]
    tile_grid.next_direction = Direction::Right;
    tile_grid.push(window_generator()); //  push [1][4][5][3][2]
    tile_grid.push(window_generator()); //  push [1][4][5][6][3][2]

    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_column(&tile_grid, 0), "Expected root node to be column");
    assert_eq!(7, tile_grid.graph.len(), "Expected 7 nodes: 6 added and 1 column parent");
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]));
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[1]));
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[2]));
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[3]));
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[4]));
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[5]));
}

#[test]
fn push_six_row_nodes() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.next_direction = Direction::Down;
    tile_grid.next_axis = SplitDirection::Horizontal;
    tile_grid.push(window_generator()); //  push [1]                 [1]
    tile_grid.push(window_generator()); //  push [1][2]              [2]
    tile_grid.push(window_generator()); //  push [1][2][3]           [3]
    tile_grid.push(window_generator()); //  push [1][2][3][4]        [4]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]     [5]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]  [6]

    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_row(&tile_grid, 0), "Expected root node to be row");
    assert_eq!(7, tile_grid.graph.len(), "Expected 7 nodes: 6 added and 1 row parent");
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]));
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[1]));
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[2]));
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[3]));
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[4]));
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[5]));
}

#[test]
fn push_six_row_nodes_altering_direction() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.next_direction = Direction::Down;
    tile_grid.next_axis = SplitDirection::Horizontal;
    tile_grid.push(window_generator()); //  push [1]                  [1]
    tile_grid.push(window_generator()); //  push [1][2]               [4]
    tile_grid.next_direction = Direction::Up; //                      [5]
    tile_grid.push(window_generator()); //  push [1][3][2]            [6]
    tile_grid.push(window_generator()); //  push [1][4][3][2]         [3]
    tile_grid.next_direction = Direction::Down; //                    [2]
    tile_grid.push(window_generator()); //  push [1][4][5][3][2]
    tile_grid.push(window_generator()); //  push [1][4][5][6][3][2]

    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_row(&tile_grid, 0), "Expected root node to be row");
    assert_eq!(7, tile_grid.graph.len(), "Expected 7 nodes: 6 added and 1 row parent");
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]));
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[1]));
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[2]));
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[3]));
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[4]));
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[5]));
}

#[test]
fn push_six_nodes_altering_axis() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.push(window_generator()); //  push [1]
    tile_grid.push(window_generator()); //  push [1][2]
    tile_grid.next_axis = SplitDirection::Horizontal;

    tile_grid.push(window_generator()); //  push [1][2]
                                           //       [1][3]

    tile_grid.push(window_generator()); //  push [1][2]
                                           //       [1][3]
                                           //       [1][4]
    tile_grid.next_axis = SplitDirection::Vertical;

    tile_grid.push(window_generator()); //  push [1][1][2][2]
                                           //       [1][1][3][3]
                                           //       [1][1][4][5]

    tile_grid.push(window_generator()); //  push [1][1][1][2][2][2]
                                           //       [1][1][1][3][3][3]
                                           //       [1][1][1][4][5][6]
    /*     
            c   
           / \
          1   r      
             /|\
            2 3 c   
               /|\
              4 5 6
    */

    let row_id = tile_grid.graph.get_sorted_children(0)[1];

    assert_eq!(0, tile_grid.graph.get_root().unwrap(), "Expected root node to take initial ID");
    assert!(is_column(&tile_grid, 0), "Expected root node to be column");
    assert_eq!(9, tile_grid.graph.len(), "Expected 9 nodes: 6 added and 2 columns and 1 row");
    assert!(is_row(&tile_grid, row_id));
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(0)[0]));

    assert_eq!(2, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_id)[0]));
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_id)[1]));

    let second_column_id = tile_grid.graph.get_sorted_children(row_id)[2];
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(second_column_id)[0]));
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(second_column_id)[1]));
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(second_column_id)[2]));
}

#[test]
fn push_twelve_nodes_altering_axis_and_directions() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, LARGE_LAYOUT);

    let root = tile_grid.graph.get_root().unwrap();
    let row_1 = tile_grid.graph.get_sorted_children(root)[1];
    let node_1 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(root)[0]);

    let node_2 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_1)[0]);
    let node_3 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_1)[1]);
    let column_1 = tile_grid.graph.get_sorted_children(row_1)[2];
    let node_5 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_1)[3]);
    let node_4 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_1)[4]);

    let node_6 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(column_1)[0]);
    let node_7 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(column_1)[1]);
    let row_2 = tile_grid.graph.get_sorted_children(column_1)[2];
    let node_9 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(column_1)[3]);
    let node_8 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(column_1)[4]);

    let node_10 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_2)[0]);
    let node_12 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_2)[1]);
    let node_11 = get_window_id(&tile_grid, tile_grid.graph.get_sorted_children(row_2)[2]);

    assert_eq!(0, root, "Expected root node to take initial ID");
    assert!(is_column(&tile_grid, root), "Expected root node to be column");
    assert!(is_column(&tile_grid, column_1));
    assert!(is_row(&tile_grid, row_1));
    assert!(is_row(&tile_grid, row_2));

    assert_eq!(1, node_1);
    assert_eq!(2, node_2);
    assert_eq!(3, node_3);
    assert_eq!(4, node_4);
    assert_eq!(5, node_5);
    assert_eq!(6, node_6);
    assert_eq!(7, node_7);
    assert_eq!(8, node_8);
    assert_eq!(9, node_9);
    assert_eq!(10, node_10);
    assert_eq!(11, node_11);
    assert_eq!(12, node_12);
}

#[test]
fn push_six_column_nodes_then_focus_each_one() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.push(window_generator()); //  push [1]
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2]
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3]
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4]
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Left);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Left);
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Left);
    tile_grid.focus(Direction::Left);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure focus stays on 1 as it's the most left column
    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // Move back to the right most column
    tile_grid.focus(Direction::Right);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure focus stays on 6 as it's the most right column
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure Up and Down have no effect as there are only columns
    tile_grid.focus(Direction::Up);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
}

#[test]
fn push_six_row_nodes_then_focus_each_one() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.next_axis = SplitDirection::Horizontal;
    tile_grid.next_direction = Direction::Down;
    tile_grid.push(window_generator()); //  push [1]
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2]
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3]
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4]
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Up);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Up);
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Up);
    tile_grid.focus(Direction::Up);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    tile_grid.focus(Direction::Up);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure focus stays on 1 as it's the top most row
    tile_grid.focus(Direction::Up);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // Move back to the bottom row
    tile_grid.focus(Direction::Down);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure focus stays on 6 as it's the bottom row
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));

    // ensure Left and Right have no effect as there are only rows
    tile_grid.focus(Direction::Left);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
}

#[test]
fn push_twelve_nodes_altering_axis_and_directions_then_focus_each_one() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, LARGE_LAYOUT);

    // Change focus around graph ensuring focus changes when it should and remains when
    // focus change in a given direction isn't allowed
    assert_eq!(12, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(9, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(8, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(8, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(3, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(4, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(5, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(7, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(10, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(12, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(11, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(7, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(6, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Left);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Up);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Down);
    assert_eq!(1, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
    tile_grid.focus(Direction::Right);
    assert_eq!(2, get_window_id(&tile_grid, tile_grid.focused_id.unwrap()));
}

#[test]
fn push_six_column_nodes_and_swap_focused() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.push(window_generator()); //  push [1]
    tile_grid.push(window_generator()); //  push [1][2]
    tile_grid.push(window_generator()); //  push [1][2][3]
    tile_grid.push(window_generator()); //  push [1][2][3][4]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]

    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[4]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[3]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[2]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[1]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[1]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[2]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[3]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[4]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
}

#[test]
fn push_six_row_nodes_and_swap_focused() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    let mut window_id = 0;
    let mut window_generator = || { window_id += 1;create_window(window_id) };
    
    tile_grid.next_direction = Direction::Down;
    tile_grid.next_axis = SplitDirection::Horizontal;
    tile_grid.push(window_generator()); //  push [1]
    tile_grid.push(window_generator()); //  push [1][2]
    tile_grid.push(window_generator()); //  push [1][2][3]
    tile_grid.push(window_generator()); //  push [1][2][3][4]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5]
    tile_grid.push(window_generator()); //  push [1][2][3][4][5][6]

    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[4]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[3]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[2]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[1]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[0]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[1]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[2]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[3]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[4]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Down);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
    tile_grid.swap_focused(Direction::Right);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(0)[5]);
}

#[test]
fn push_twelve_nodes_altering_axis_and_directions_then_swap_focused_around() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, LARGE_LAYOUT);

    let root = tile_grid.graph.get_root().unwrap();
    let row_1 = tile_grid.graph.get_sorted_children(root)[1];
    let column_1 = tile_grid.graph.get_sorted_children(row_1)[2];
    let row_2 = tile_grid.graph.get_sorted_children(column_1)[2];

    // Reference large layout. 
    // start at 12's position
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(row_2)[1]);
    tile_grid.move_focused_out(Direction::Up);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(row_1)[2]);
    tile_grid.move_focused_out(Direction::Left);
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(root)[1]);
    tile_grid.move_focused_in(Direction::Left);
    let new_row = tile_grid.graph.get_sorted_children(root)[0];
    assert_eq!(tile_grid.focused_id.unwrap(), tile_grid.graph.get_sorted_children(new_row)[1]);
    tile_grid.move_focused_out(Direction::Up);
}

#[test]
fn make_space_for_node_test_check_size_distributions() {
    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(120, tile_grid.graph.node(node_id).get_size());
        }
    }

    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p,p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(60, tile_grid.graph.node(node_id).get_size());
        }
    }

    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p,p,p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(40, tile_grid.graph.node(node_id).get_size());
        }
    }

    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p,p,p,p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(30, tile_grid.graph.node(node_id).get_size());
        }
    }

    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p,p,p,p,p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(24, tile_grid.graph.node(node_id).get_size());
        }
    }

    let mut tile_grid = TileGrid::new(0, TestRenderer { } );
    perform_actions(&mut tile_grid, "p,p,p,p,p,p");
    for node_id in tile_grid.graph.nodes() {
        if tile_grid.graph.node(node_id).is_tile() {
            assert_eq!(20, tile_grid.graph.node(node_id).get_size());
        }
    }
}

fn print(tile_grid: &TileGrid) {
    let render_infos = tile_grid.get_render_info(127, 90);
    println!("{}", TextRenderer::render(127, 90, render_infos)); 
}

struct TestRenderer {}

impl Renderer for TestRenderer {
    fn render<TRenderer: Renderer>(
        &self,
        grid: &TileGrid<TRenderer>,
        window: &NativeWindow,
        config: &Config,
        display: &Display,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> SystemResult {
        Ok(())
    }
}
