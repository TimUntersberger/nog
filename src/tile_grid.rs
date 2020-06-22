use crate::app_bar;
use crate::config::Direction;
use crate::config::FocusDirection;
use crate::config::SwapDirection;
use crate::tile::Tile;
use crate::util;
use crate::window::Window;
use crate::CONFIG;
use log::debug;
use winapi::shared::windef::HWND;
use winapi::um::winuser::SetWindowPos;

#[derive(Clone, EnumString, Copy)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct TileGrid {
    pub id: i32,
    pub visible: bool,
    pub focus_stack: Vec<(FocusDirection, i32)>,
    pub tiles: Vec<Tile>,
    pub focused_window_id: Option<i32>,
    pub taskbar_window: i32,
    pub rows: i32,
    pub columns: i32,
    pub height: i32,
    pub width: i32,
}

impl TileGrid {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            visible: false,
            tiles: Vec::new(),
            focus_stack: Vec::with_capacity(5),
            focused_window_id: None,
            taskbar_window: 0,
            rows: 0,
            columns: 0,
            height: 0,
            width: 0,
        }
    }
    pub fn hide(&mut self) {
        for tile in &self.tiles {
            tile.window.hide();
        }
        self.visible = false;
    }
    pub fn show(&mut self) {
        for tile in &self.tiles {
            tile.window.show();
            tile.window.to_foreground(true);
            tile.window.remove_topmost();
        }
        if let Some(tile) = self.get_focused_tile() {
            tile.window.focus();
        }
        self.visible = true;
    }
    pub fn get_tile_by_id(&self, id: i32) -> Option<Tile> {
        self.tiles
            .iter()
            .find(|tile| tile.window.id == id)
            .map(|t| t.clone())
    }
    pub fn get_tile_by_id_mut(&mut self, id: i32) -> Option<&mut Tile> {
        self.tiles.iter_mut().find(|tile| tile.window.id == id)
    }
    pub fn get_focused_tile(&self) -> Option<&Tile> {
        return self
            .focused_window_id
            .and_then(|id| self.tiles.iter().find(|tile| tile.window.id == id));
    }
    pub fn get_focused_tile_mut(&mut self) -> Option<&mut Tile> {
        return self
            .focused_window_id
            .and_then(move |id| self.tiles.iter_mut().find(|tile| tile.window.id == id));
    }
    pub fn set_focused_split_direction(&mut self, direction: SplitDirection) {
        if let Some(focused_tile) = self.get_focused_tile_mut() {
            focused_tile.split_direction = direction;
        }
    }
    fn get_next_tile_id(&self, direction: FocusDirection) -> Option<i32> {
        self.get_next_tile(direction).map(|t| t.window.id)
    }
    fn get_next_tile(&self, direction: FocusDirection) -> Option<Tile> {
        self.get_focused_tile().and_then(|focused_tile| {
            //Whether it is possible to go in that direction or not
            let possible = !match direction {
                FocusDirection::Right => {
                    focused_tile.column == Some(self.columns) || focused_tile.column == None
                }
                FocusDirection::Left => {
                    focused_tile.column == Some(1) || focused_tile.column == None
                }
                FocusDirection::Up => focused_tile.row == Some(1) || focused_tile.row == None,
                FocusDirection::Down => {
                    focused_tile.row == Some(self.rows) || focused_tile.row == None
                }
            };

            if !possible {
                debug!("It is not possible to focus in this direction");
                return None;
            }

            debug!("It is possible to focus in this direction");

            self.tiles
                .iter()
                .find(|tile| match direction {
                    FocusDirection::Right => {
                        (focused_tile.row == None
                            || tile.row == None
                            || tile.row == focused_tile.row)
                            && tile.column == focused_tile.column.map(|x| x + 1)
                        // && (tile.row == Some(1) || tile.row == None)
                    }
                    FocusDirection::Left => {
                        (focused_tile.row == None
                            || tile.row == None
                            || tile.row == focused_tile.row)
                            && tile.column == focused_tile.column.map(|x| x - 1)
                        // && (tile.row == Some(1) || tile.row == None)
                    }
                    FocusDirection::Up => {
                        (focused_tile.column == None
                            || tile.column == None
                            || tile.column == focused_tile.column)
                            && tile.row == focused_tile.row.map(|x| x - 1)
                        // && (tile.column == Some(1) || tile.column == None)
                    }
                    FocusDirection::Down => {
                        (focused_tile.column == None
                            || tile.column == None
                            || tile.column == focused_tile.column)
                            && tile.row == focused_tile.row.map(|x| x + 1)
                        // && (tile.column == Some(1) || tile.column == None)
                    }
                })
                .map(|t| t.clone())
        })
    }
    fn set_location(&mut self, id: i32, row: Option<i32>, col: Option<i32>) {
        if let Some(mut tile) = self.get_tile_by_id_mut(id) {
            tile.row = row;
            tile.column = col;
        }
    }
    fn swap_tiles(&mut self, x: i32, y: i32) {
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
    pub fn swap(&mut self, direction: SwapDirection) -> Result<(), util::WinApiResultError> {
        if let Some(tile) = self.check_focus_stack(direction)? {
            //if the focus stack is not empty, then some tile must have focus
            let focused_id = self.focused_window_id.unwrap();
            self.swap_tiles(tile.window.id, focused_id);
            return Ok(());
        }
        let maybe_next_id = self.get_next_tile_id(direction);
        if maybe_next_id.is_some() {
            let next_id = maybe_next_id.unwrap();
            //if we get a next tile we can assume that a tile is focused
            let focused_id = self.focused_window_id.unwrap();
            self.swap_tiles(next_id, focused_id);
            self.focus_stack.push((direction, next_id));
        }

        Ok(())
    }
    fn check_focus_stack(
        &mut self,
        direction: Direction,
    ) -> Result<Option<Tile>, util::WinApiResultError> {
        if let Some(prev) = self.focus_stack.pop() {
            // This variable says that the action cancels the previous action.
            // Example: Left -> Right
            let counters = match direction {
                FocusDirection::Left => prev.0 == FocusDirection::Right,
                FocusDirection::Right => prev.0 == FocusDirection::Left,
                FocusDirection::Up => prev.0 == FocusDirection::Down,
                FocusDirection::Down => prev.0 == FocusDirection::Up,
            };

            if counters {
                let maybe_tile = self.get_tile_by_id(prev.1);

                if maybe_tile.is_some() {
                    debug!("The direction counters the previous one. Reverting the previous one.");
                    let tile = maybe_tile.unwrap();
                    return Ok(Some(tile));
                }
            }

            self.focus_stack.push(prev);

            if self.focus_stack.len() == self.focus_stack.capacity() {
                debug!("Focus stack exceeded the limit. Removing oldest one");
                self.focus_stack.drain(0..1);
            }
        }

        return Ok(None);
    }
    pub fn focus(&mut self, direction: FocusDirection) -> Result<(), util::WinApiResultError> {
        //TODO: Fix no split -> split | no focus stack
        if let Some(tile) = self.check_focus_stack(direction)? {
            self.focused_window_id = Some(tile.window.id);
            tile.window.focus()?;
            return Ok(());
        }

        let maybe_next_tile = self.get_next_tile(direction);

        if maybe_next_tile.is_some() {
            let next_tile = maybe_next_tile.unwrap();

            self.focus_stack
                .push((direction, self.focused_window_id.unwrap()));

            self.focused_window_id = Some(next_tile.window.id);
            next_tile.window.focus()?;
        }

        debug!("Couldn't find a valid tile");

        Ok(())
    }
    pub fn focus_right(&mut self) -> Result<(), util::WinApiResultError> {
        self.focus(FocusDirection::Right)
    }
    pub fn focus_left(&mut self) -> Result<(), util::WinApiResultError> {
        self.focus(FocusDirection::Left)
    }
    pub fn focus_up(&mut self) -> Result<(), util::WinApiResultError> {
        self.focus(FocusDirection::Up)
    }
    pub fn focus_down(&mut self) -> Result<(), util::WinApiResultError> {
        self.focus(FocusDirection::Down)
    }
    pub fn close_tile_by_window_id(&mut self, id: i32) -> Option<Tile> {
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

                self.rows = self.rows - 1;
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

                self.columns = self.columns - 1;

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
                            t.column.map(|x| x - 1);
                        })
                }
            } else {
                let mut tiles_in_column: Vec<&mut Tile> = self.tiles
                    .iter_mut()
                    .filter(|t| t.column == removed_tile.column)
                    .collect();

                let tile_count = tiles_in_column.len();

                println!("{}", tile_count);
                
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

            if self.tiles.len() == 0 {
                self.focused_window_id = None;
            } else if let Some(focused_window_id) = self.focused_window_id {
                if focused_window_id == removed_tile.window.id {
                    let next_column = removed_tile.column.map(|column| {
                        return if column > self.columns {
                            column - 1
                        } else {
                            column
                        };
                    });

                    let next_row = removed_tile
                        .row
                        .map(|row| return if row > self.rows { row - 1 } else { row });

                    let maybe_next_tile: Option<&Tile> = self.tiles.iter().find(|tile| {
                        return tile.column == next_column && tile.row == next_row;
                    });

                    if let Some(next_tile) = maybe_next_tile {
                        self.focused_window_id = Some(next_tile.window.id);
                    }
                }
            }
        }

        return maybe_removed_tile;
    }
    pub fn split(&mut self, window: Window) {
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

                        // We can assume that row is Some because, there is no case where it is currently None
                        if row.unwrap() <= self.rows {
                            self.tiles
                                .iter_mut()
                                .filter(|t| t.column == column && t.row >= row)
                                .for_each(|t| {
                                    t.row = t.row.map(|x| x + 1);
                                });
                        }
                        if row > Some(self.rows) {
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

                        // We can assume that column is Some because, there is no case where it is currently None
                        if column.unwrap() <= self.columns {
                            self.tiles
                                .iter_mut()
                                .filter(|t| t.row == row && t.column >= column)
                                .for_each(|t| {
                                    t.column = t.column.map(|x| x + 1);
                                });
                        }

                        if column > Some(self.columns) {
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
                    ..Tile::default()
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
    /// Calculates all the data required for drawing the tile
    /// # return
    /// (x, y, width, height)
    fn calculate_tile_data(&self, tile: &Tile) -> (i32, i32, i32, i32) {
        let column_width = self.width / self.columns;
        let row_height = self.height / self.rows;

        // These deltas are mainly required for resizing/moving windows with a native titlebar
        // I don't know why, but the coordinates are different when a window has a native titlebar :shrug:
        let mut column_delta = 0;
        let mut row_delta = 0;
        let mut x = 0;
        let mut y = 0;
        let mut height = self.height;
        let mut width = self.width;

        if let Some(column) = tile.column {
            if !CONFIG.remove_title_bar && column > 1 {
                column_delta = 15;
            }

            width = column_width + column_delta;
            x = column_width * (column - 1) - column_delta;

            width -= CONFIG.padding / self.columns;

            if column > 1 {
                x += CONFIG.padding / self.columns;
            }
        }

        if let Some(row) = tile.row {
            if !CONFIG.remove_title_bar && row > 1 {
                row_delta = 10;
            }

            height = row_height + row_delta;
            y = row_height * (row - 1) - row_delta;

            height -= CONFIG.padding / self.rows;

            if row > 1 {
                y += CONFIG.padding / self.rows;
            }
        }

        if !CONFIG.remove_title_bar {
            x -= 8;
            y -= 1;
        } else if let Some(rule) = &tile.window.rule {
            if rule.has_custom_titlebar {
                x += rule.x;
                width += rule.width;
            }
        }

        x += CONFIG.margin;
        y += CONFIG.margin;
        y += *app_bar::HEIGHT.lock().unwrap();

        (x, y, width, height)
    }

    fn draw_tile(&self, tile: &Tile) {
        let (x, y, width, height) = self.calculate_tile_data(tile);

        unsafe {
            //TODO: handle error
            SetWindowPos(
                tile.window.id as HWND,
                std::ptr::null_mut(),
                x,
                y,
                width,
                height,
                0,
            );
        }
    }

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

        print!("\n");

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
            print!("\n");
        }

        print!("\n");
    }

    pub fn draw_grid(&self) {
        debug!("Drawing grid");

        for tile in &self.tiles {
            debug!("row: {:?} column: {:?}", tile.row, tile.column);

            self.draw_tile(tile);
        }

        // self.print_grid();
    }
}
