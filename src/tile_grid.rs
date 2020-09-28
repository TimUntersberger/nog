use crate::{
    direction::Direction,
    display::{get_primary_display, Display},
    renderer::{NativeRenderer, Renderer},
    split_direction::SplitDirection,
    system::NativeWindow,
    system::SystemError,
    system::SystemResult,
    system::WindowId,
    tile::Tile,
};
use log::{debug, error};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TileGrid<TRenderer: Renderer = NativeRenderer> {
    pub display: Display,
    pub renderer: TRenderer,
    pub id: i32,
    pub fullscreen: bool,
    pub focus_stack: Vec<(Direction, WindowId)>,
    /// Contains the resize values for each column
    /// Hashmap<column, (left, right)>
    pub column_modifications: HashMap<i32, (i32, i32)>,
    /// Contains the resize values for each row
    /// Hashmap<column, (top, bottom)>
    pub row_modifications: HashMap<i32, (i32, i32)>,
    pub tiles: Vec<Tile>,
    pub focused_window_id: Option<WindowId>,
    pub taskbar_window: i32,
    pub rows: i32,
    pub columns: i32,
}

impl<TRenderer: Renderer> TileGrid<TRenderer> {
    pub fn new(id: i32, renderer: TRenderer) -> TileGrid<TRenderer> {
        Self {
            id,
            display: get_primary_display(),
            renderer,
            fullscreen: false,
            tiles: Vec::new(),
            focus_stack: Vec::with_capacity(5),
            column_modifications: HashMap::new(),
            row_modifications: HashMap::new(),
            focused_window_id: None,
            taskbar_window: 0,
            rows: 0,
            columns: 0,
        }
    }
    pub fn hide(&self) {
        for tile in &self.tiles {
            tile.window.hide();
        }
    }
    pub fn toggle_fullscreen(&mut self) {
        if self.fullscreen || !self.tiles.is_empty() {
            self.fullscreen = !self.fullscreen;
            self.draw_grid();
        }
    }
    pub fn show(&self) -> SystemResult {
        for tile in &self.tiles {
            tile.window.show();
            tile.window
                .to_foreground(true)
                .map_err(SystemError::ShowWindow)?;
            if let Err(e) = tile.window.remove_topmost() {
                error!("{}", e);
            }
        }
        if let Some(tile) = self.get_focused_tile() {
            tile.window.focus()?;
        }
        Ok(())
    }
    pub fn get_tile_by_id(&self, id: WindowId) -> Option<Tile> {
        self.tiles
            .iter()
            .find(|tile| tile.window.id == id)
            .clone()
            .cloned()
    }
    pub fn get_tile_by_id_mut(&mut self, id: WindowId) -> Option<&mut Tile> {
        self.tiles.iter_mut().find(|tile| tile.window.id == id)
    }
    pub fn get_focused_tile(&self) -> Option<&Tile> {
        self.focused_window_id
            .and_then(|id| self.tiles.iter().find(|tile| tile.window.id == id))
    }
    pub fn get_focused_tile_mut(&mut self) -> Option<&mut Tile> {
        self.focused_window_id
            .and_then(move |id| self.tiles.iter_mut().find(|tile| tile.window.id == id))
    }
    pub fn set_focused_split_direction(&mut self, direction: SplitDirection) {
        if let Some(focused_tile) = self.get_focused_tile_mut() {
            focused_tile.split_direction = direction;
        }
    }
    fn get_next_tile_id(&self, direction: Direction) -> Option<WindowId> {
        self.get_next_tile(direction).map(|t| t.window.id)
    }
    fn get_next_tile(&self, direction: Direction) -> Option<Tile> {
        self.get_focused_tile().and_then(|focused_tile| {
            //Whether it is possible to go in that direction or not
            let possible = !match direction {
                Direction::Right => {
                    focused_tile.column == Some(self.columns) || focused_tile.column == None
                }
                Direction::Left => focused_tile.column == Some(1) || focused_tile.column == None,
                Direction::Up => focused_tile.row == Some(1) || focused_tile.row == None,
                Direction::Down => focused_tile.row == Some(self.rows) || focused_tile.row == None,
            };

            if !possible {
                debug!("It is not possible to focus in this direction");
                return None;
            }

            debug!("It is possible to focus in this direction");

            self.tiles
                .iter()
                .find(|tile| match direction {
                    Direction::Right => {
                        (focused_tile.row == None
                            || tile.row == None
                            || tile.row == focused_tile.row)
                            && tile.column == focused_tile.column.map(|x| x + 1)
                        // && (tile.row == Some(1) || tile.row == None)
                    }
                    Direction::Left => {
                        (focused_tile.row == None
                            || tile.row == None
                            || tile.row == focused_tile.row)
                            && tile.column == focused_tile.column.map(|x| x - 1)
                        // && (tile.row == Some(1) || tile.row == None)
                    }
                    Direction::Up => {
                        (focused_tile.column == None
                            || tile.column == None
                            || tile.column == focused_tile.column)
                            && tile.row == focused_tile.row.map(|x| x - 1)
                        // && (tile.column == Some(1) || tile.column == None)
                    }
                    Direction::Down => {
                        (focused_tile.column == None
                            || tile.column == None
                            || tile.column == focused_tile.column)
                            && tile.row == focused_tile.row.map(|x| x + 1)
                        // && (tile.column == Some(1) || tile.column == None)
                    }
                })
                .cloned()
        })
    }
    fn set_location(&mut self, id: WindowId, row: Option<i32>, col: Option<i32>) {
        if let Some(mut tile) = self.get_tile_by_id_mut(id) {
            tile.row = row;
            tile.column = col;
        }
    }
    fn swap_tiles(&mut self, x: WindowId, y: WindowId) {
        //borrow checker bullshit
        let x_tile = {
            let tile = self.get_tile_by_id(x).unwrap();
            (tile.window.id, tile.row, tile.column)
        };
        let y_tile = {
            let tile = self.get_tile_by_id(y).unwrap();
            (tile.window.id, tile.row, tile.column)
        };
        self.set_location(x_tile.0, y_tile.1, y_tile.2);
        self.set_location(y_tile.0, x_tile.1, x_tile.2);
    }
    pub fn swap(&mut self, direction: Direction) {
        if let Some(tile) = self.check_focus_stack(direction) {
            //if the focus stack is not empty, then some tile must have focus
            let focused_id = self.focused_window_id.unwrap();
            self.swap_tiles(tile.window.id, focused_id);
        } else if let Some(next_id) = self.get_next_tile_id(direction) {
            //if we get a next tile we can assume that a tile is focused
            let focused_id = self.focused_window_id.unwrap();
            self.swap_tiles(next_id, focused_id);
            self.focus_stack.push((direction, next_id));
        }
    }
    fn check_focus_stack(&mut self, direction: Direction) -> Option<Tile> {
        if let Some(prev) = self.focus_stack.pop() {
            // This variable says that the action cancels the previous action.
            // Example: Left -> Right
            let counters = match direction {
                Direction::Left => prev.0 == Direction::Right,
                Direction::Right => prev.0 == Direction::Left,
                Direction::Up => prev.0 == Direction::Down,
                Direction::Down => prev.0 == Direction::Up,
            };

            if counters {
                let maybe_tile = self.get_tile_by_id(prev.1);

                if let Some(tile) = maybe_tile {
                    debug!("The direction counters the previous one. Reverting the previous one.");
                    return Some(tile);
                }
            }

            self.focus_stack.push(prev);

            if self.focus_stack.len() == self.focus_stack.capacity() {
                debug!("Focus stack exceeded the limit. Removing oldest one");
                self.focus_stack.drain(0..1);
            }
        }

        None
    }
    pub fn focus(&mut self, direction: Direction) -> SystemResult {
        if let Some(tile) = self.check_focus_stack(direction) {
            self.focused_window_id = Some(tile.window.id);
            tile.window.focus()?;
            return Ok(());
        }

        let maybe_next_tile = self.get_next_tile(direction);

        if let Some(next_tile) = maybe_next_tile {
            self.focus_stack
                .push((direction, self.focused_window_id.unwrap()));

            self.focused_window_id = Some(next_tile.window.id);
            next_tile.window.focus()?;
        } else {
            debug!("Couldn't find a valid tile");
        }

        Ok(())
    }
    pub fn focus_right(&mut self) -> SystemResult {
        self.focus(Direction::Right)
    }
    pub fn focus_left(&mut self) -> SystemResult {
        self.focus(Direction::Left)
    }
    pub fn focus_up(&mut self) -> SystemResult {
        self.focus(Direction::Up)
    }
    pub fn focus_down(&mut self) -> SystemResult {
        self.focus(Direction::Down)
    }
    pub fn close_tile_by_window_id(&mut self, id: WindowId) -> Option<Tile> {
        let maybe_removed_tile = self
            .tiles
            .iter()
            .position(|tile| tile.window.id == id)
            .map(|idx| self.tiles.remove(idx));

        if let Some(removed_tile) = maybe_removed_tile.clone() {
            let is_empty_row = removed_tile.row != None
                && !self.tiles.iter().any(|tile| tile.row == removed_tile.row);

            let is_empty_column = removed_tile.column != None
                && !self
                    .tiles
                    .iter()
                    .any(|tile| tile.column == removed_tile.column);

            if is_empty_row {
                debug!("row is now empty");

                self.row_modifications.remove(&self.rows);
                self.rows -= 1;
                if self.rows == 1 {
                    self.tiles
                        .iter_mut()
                        .filter(|t| t.row > removed_tile.row)
                        .for_each(|t| {
                            t.row = None;
                        });
                } else {
                    self.tiles
                        .iter_mut()
                        .filter(|t| t.row > removed_tile.row)
                        .for_each(|t| {
                            t.row.map(|x| x - 1);
                        });
                }
            } else if !is_empty_column {
                self.tiles
                    .iter_mut()
                    .filter(|t| t.column == removed_tile.column)
                    .for_each(|t| t.row = None);
            }

            if is_empty_column {
                debug!("column is now empty");

                self.column_modifications.remove(&self.columns);
                self.columns -= 1;

                if self.columns == 1 {
                    self.tiles
                        .iter_mut()
                        .filter(|t| t.column != removed_tile.column)
                        .for_each(|t| {
                            t.column = None;
                        })
                } else {
                    self.tiles
                        .iter_mut()
                        .filter(|t| t.column > removed_tile.column)
                        .for_each(|t| {
                            t.column = t.column.map(|x| x - 1);
                        })
                }
            } else {
                let mut tiles_in_column: Vec<&mut Tile> = self
                    .tiles
                    .iter_mut()
                    .filter(|t| t.column == removed_tile.column)
                    .collect();

                let tile_count = tiles_in_column.len();

                for t in tiles_in_column.iter_mut() {
                    t.row = if tile_count == 1 {
                        None
                    } else if removed_tile.row < t.row {
                        t.row.map(|x| x - 1)
                    } else {
                        t.row
                    };
                }
            }

            if self.tiles.is_empty() {
                self.focused_window_id = None;
            } else if let Some(focused_window_id) = self.focused_window_id {
                if focused_window_id == removed_tile.window.id {
                    let next_column = removed_tile.column.map(|column| {
                        if column > self.columns {
                            column - 1
                        } else {
                            column
                        }
                    });

                    let next_row =
                        removed_tile
                            .row
                            .map(|row| if row > self.rows { row - 1 } else { row });

                    let maybe_next_tile: Option<&Tile> = self.tiles.iter().find(|tile| {
                        (tile.column == None || tile.column == next_column)
                            && (tile.row == None || tile.row == next_row)
                    });

                    if let Some(next_tile) = maybe_next_tile {
                        self.focused_window_id = Some(next_tile.window.id);
                    }
                }
            }
        }

        maybe_removed_tile
    }
    pub fn split(&mut self, window: NativeWindow) {
        if self.tiles.iter().any(|t| t.window.id == window.id) {
            return;
        }

        match self.get_focused_tile_mut() {
            Some(focused_tile) => {
                let split_direction = focused_tile.split_direction;
                let (column, row) = match focused_tile.split_direction {
                    SplitDirection::Horizontal => {
                        if focused_tile.row == None {
                            focused_tile.row = Some(1);
                        }

                        let column = focused_tile.column;
                        let row = focused_tile.row.map(|x| x + 1).or(Some(2));

                        // This is not 0 when the new row is not the last one
                        // It basically is the count of rows in this row that come after the location where this new tile gets placed
                        let mut row_count = 0;

                        // We can assume that row is Some because, there is no case where it is currently None
                        if row.unwrap() <= self.rows {
                            self.tiles
                                .iter_mut()
                                .filter(|t| t.column == column && t.row >= row)
                                .for_each(|t| {
                                    t.row = t.row.map(|x| x + 1);
                                    row_count += 1;
                                });
                        }
                        if row.map(|x| x + row_count) > Some(self.rows) {
                            self.rows += 1;
                        }

                        (column, row)
                    }
                    SplitDirection::Vertical => {
                        if focused_tile.column == None {
                            focused_tile.column = Some(1);
                        }
                        let row = focused_tile.row;
                        let column = focused_tile.column.map(|x| x + 1).or(Some(2));

                        // This is not 0 when the new column is not the last one
                        // It basically is the count of columns in this row that come after the location where this new tile gets placed
                        let mut column_count = 0;

                        // We can assume that column is Some because, there is no case where it is currently None
                        if column.unwrap() <= self.columns {
                            self.tiles
                                .iter_mut()
                                .filter(|t| t.row == row && t.column >= column)
                                .for_each(|t| {
                                    t.column = t.column.map(|x| x + 1);
                                    column_count += 1;
                                });
                        }

                        if column.map(|x| x + column_count) > Some(self.columns) {
                            self.columns += 1;
                        }

                        (column, row)
                    }
                };

                self.focused_window_id = Some(window.id);
                self.tiles.push(Tile {
                    row,
                    column,
                    split_direction,
                    window,
                });
            }
            None => {
                self.rows = 1;
                self.columns = 1;
                self.focused_window_id = Some(window.id);
                self.tiles.push(Tile {
                    window,
                    ..Tile::default()
                });
            }
        }
    }

    pub fn get_row_modifications(&self, row: Option<i32>) -> Option<(i32, i32)> {
        row.and_then(|value| self.row_modifications.get(&value))
            .copied()
    }
    pub fn get_column_modifications(&self, column: Option<i32>) -> Option<(i32, i32)> {
        column
            .and_then(|value| self.column_modifications.get(&value))
            .copied()
    }
    pub fn get_modifications(&self, tile: &Tile) -> (Option<(i32, i32)>, Option<(i32, i32)>) {
        (
            self.get_column_modifications(tile.column),
            self.get_row_modifications(tile.row),
        )
    }
    /// Converts the percentage to the real pixel value of the current display
    pub fn percentage_to_real(&self, p: i32) -> i32 {
        self.display.working_area_height() / 100 * p
    }

    pub fn resize_column(&mut self, column: i32, direction: Direction, amount: i32) {
        if amount != 0 {
            let mut modification = self
                .get_column_modifications(Some(column))
                .unwrap_or((0, 0));

            if direction == Direction::Left && column != 1 {
                modification.0 += amount;
            } else if direction == Direction::Right && column != self.columns {
                modification.1 += amount;
            }

            self.column_modifications.insert(column, modification);
        }
    }

    pub fn resize_row(&mut self, row: i32, direction: Direction, amount: i32) {
        if amount != 0 {
            let mut modification = self.get_row_modifications(Some(row)).unwrap_or((0, 0));

            if direction == Direction::Up && row != 1 {
                modification.0 += amount;
            } else if direction == Direction::Down && row != self.rows {
                modification.1 += amount;
            }

            self.row_modifications.insert(row, modification);
        }
    }

    #[allow(dead_code)]
    fn print_grid(&self) {
        debug!("Printing grid");

        if self.rows == 0 || self.columns == 0 {
            print!("\nEmpty\n\n");
            return;
        }

        let mut rows = [[std::ptr::null(); 10]; 10];

        //TODO: Add checks for safety
        //      Example:
        //          Invalid row or column on tile
        //      Without any checks we might get a STATUS_ACCESS_VIOLATION error from windows, which can't be handled by us.

        for tile in &self.tiles {
            match tile.row {
                Some(row) => match tile.column {
                    Some(column) => rows[(row - 1) as usize][(column - 1) as usize] = &tile.window,
                    None => {
                        for i in 0..self.columns {
                            rows[(row - 1) as usize][i as usize] = &tile.window;
                        }
                    }
                },
                None => match tile.column {
                    Some(column) => {
                        for i in 0..self.rows {
                            rows[i as usize][(column - 1) as usize] = &tile.window;
                        }
                    }
                    None => rows[0][0] = &tile.window,
                },
            }
        }

        println!();

        for row in 0..self.rows {
            print!("|");
            for column in 0..self.columns {
                unsafe {
                    let window = &(*rows[row as usize][column as usize]);
                    if let Some(id) = self.focused_window_id {
                        match window.id == id {
                            true => print!("* {}({}) *|", window.title, window.id),
                            false => print!(" {}({}) |", window.title, window.id),
                        }
                    }
                }
            }
            println!();
        }

        println!();
    }

    pub fn draw_grid(&self) -> SystemResult {
        debug!("Drawing grid");

        if self.fullscreen {
            if let Some(tile) = self.get_focused_tile() {
                self.renderer.render(self, tile)?;
            }
        } else {
            for tile in &self.tiles {
                debug!("{:?}", tile);

                self.renderer.render(self, tile)?;
            }

            // self.print_grid();
        }

        Ok(())
    }
}
