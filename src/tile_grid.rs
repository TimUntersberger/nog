use crate::CONFIG;
use crate::tile::Tile;
use crate::window::Window;
use crate::util;
use crate::app_bar;
use winapi::shared::windef::HWND;
use winapi::um::winuser::SetWindowPos;

#[derive(Clone, EnumString)]
pub enum SplitDirection {
    Horizontal,
    Vertical
}

// TODO: A TileGrid will need a last focus stack where each item has a direction and window id.
// When a focus function gets called peek at the last focus stack to know whether the sequence cancels itself to pop the last item.

// the stack will need maximum limit. right now im thinking about like 5 items max?

//TODO(#20)
#[derive(Clone)]
pub struct TileGrid {
    pub id: i32,
    pub visible: bool,
    pub tiles: Vec<Tile>,
    pub focused_window_id: Option<i32>,
    pub taskbar_window: i32,
    pub rows: i32,
    pub columns: i32,
    pub height: i32,
    pub width: i32
}

impl TileGrid {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            visible: false,
            tiles: Vec::<Tile>::new(),
            focused_window_id: None,
            taskbar_window: 0,
            rows: 0,
            columns: 0,
            height: 0,
            width: 0
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
    pub fn get_focused_tile(&self) -> Option<&Tile> {
        return self.focused_window_id
            .and_then(|id| self.tiles
                .iter()
                .find(|tile| tile.window.id == id));
    }
    pub fn get_focused_tile_mut(&mut self) -> Option<&mut Tile> {
        return self.focused_window_id
            .and_then(move |id| self.tiles
                .iter_mut()
                .find(|tile| tile.window.id == id));
    }
    pub fn set_focused_split_direction(&mut self, direction: SplitDirection) {
        if let Some(focused_tile) = self.get_focused_tile_mut() {
            focused_tile.split_direction = direction;
        }
    }
    pub fn focus_right(&mut self) -> Result<(), util::WinApiResultError>{
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.column == Some(self.columns) || focused_tile.column == None {
                return Ok(());
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.row == None || tile.row == focused_tile.row) && tile.column == focused_tile.column.map(|x| x + 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                next_tile.window.focus()?;
            }
        }

        Ok(())
    }
    pub fn focus_left(&mut self) -> Result<(), util::WinApiResultError>{
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.column == Some(1) || focused_tile.column == None {
                return Ok(());
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.row == None || tile.row == focused_tile.row) && tile.column == focused_tile.column.map(|x| x - 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                next_tile.window.focus()?;
            }
        }

        Ok(())
    }
    pub fn focus_up(&mut self) -> Result<(), util::WinApiResultError>{
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.row == Some(1) || focused_tile.row == None {
                return Ok(());
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.column == None || tile.column == focused_tile.column) && tile.row == focused_tile.row.map(|x| x - 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                next_tile.window.focus()?;
            }
        }

        Ok(())
    }
    pub fn focus_down(&mut self) -> Result<(), util::WinApiResultError>{
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.row == Some(self.rows) || focused_tile.row == None {
                return Ok(());
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.column == None || tile.column == focused_tile.column) && tile.row == focused_tile.row.map(|x| x + 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                next_tile.window.focus()?;
            }
        }

        Ok(())
    }
    pub fn close_tile_by_window_id(&mut self, id: i32) -> Option<Tile> {
        let maybe_removed_tile = self.tiles
            .iter()
            .position(|tile| tile.window.id == id)
            .map(|idx| self.tiles.remove(idx));

        if let Some(removed_tile) = maybe_removed_tile.clone() {
            let is_empty_row = !self.tiles
                .iter()
                .any(|tile| tile.row == removed_tile.row);

            let is_empty_column = !self.tiles
                .iter()
                .any(|tile| tile.column == removed_tile.column);

            if is_empty_row {
                self.rows = self.rows - 1;
                let tiles_after_deleted_tile = self.tiles
                    .iter_mut()
                    .filter(|t| t.row > removed_tile.row);

                for tile in tiles_after_deleted_tile {
                    tile.row = tile.row.map(|x| x - 1);
                }
            }

            if is_empty_column {
                self.columns = self.columns - 1;
                let tiles_after_deleted_tile = self.tiles
                    .iter_mut()
                    .filter(|t| t.column > removed_tile.column);

                for tile in tiles_after_deleted_tile {
                    tile.column = tile.column.map(|x| x - 1);
                }
            }

            if self.tiles.len() == 0 {
                self.focused_window_id = None;
            }
            else if let Some(focused_window_id) = self.focused_window_id {
                if focused_window_id == removed_tile.window.id {
                    let next_column = removed_tile.column.map(|column| {
                        return if column > self.columns {
                            column - 1
                        } else {
                            column
                        }
                    });

                    let next_row = removed_tile.row.map(|row| {
                        return if row > self.rows {
                            row - 1
                        } else {
                            row
                        }
                    });

                    let maybe_next_tile: Option<&Tile> = self.tiles
                        .iter()
                        .find(|tile| {
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
    pub fn split(&mut self, window: Window){
        if self.tiles.iter().any(|t| t.window.id == window.id) {
            return;
        }

        match self.get_focused_tile_mut() {
            Some(focused_tile) => {
                let mut new_tile = Tile::new(window);

                match focused_tile.split_direction {
                    SplitDirection::Horizontal => {
                        new_tile.column = focused_tile.column;
                        match focused_tile.row {
                            Some(row) => new_tile.row = Some(row + 1),
                            None => {
                                focused_tile.row = Some(1);
                                new_tile.row = Some(2);
                            }
                        }
                        self.rows = self.rows + 1;
                    },
                    SplitDirection::Vertical => {
                        new_tile.row = focused_tile.row;
                        match focused_tile.column {
                            Some(column) => new_tile.column = Some(column + 1),
                            None => {
                                focused_tile.column = Some(1);
                                new_tile.column = Some(2);
                            }
                        }
                        self.columns = self.columns + 1;
                    }
                }

                self.focused_window_id = Some(new_tile.window.id);
                self.tiles.push(new_tile);
            },
            None => {
                self.rows = 1;
                self.columns = 1;
                self.focused_window_id = Some(window.id);
                self.tiles.push(Tile::new(window));
            } 
        }
    }
    fn draw_tile_with_title_bar(&self, tile: &Tile) {
        let column_width = self.width / self.columns;
        let row_height = self.height / self.rows;

        let column_delta = match tile.column {
            Some(column) => if column > 1 {
                15
            } else {
                0
            },
            None => 0
        };

        let row_delta = match tile.row {
            Some(row) => if row > 1 {
                10
            } else {
                0
            },
            None => 0
        };

        let x = match tile.column {
            Some(column) => column_width * (column - 1) - 8 - column_delta,
            None => -8
        };

        let y = match tile.row {
            Some(row) => row_height * (row - 1) - row_delta - 1,
            None => -1
        };

        let height = match tile.row {
            Some(_row) => row_height + row_delta,
            None => self.height
        };

        let width = match tile.column {
            Some(_column) => column_width + column_delta,
            None => self.width
        };

        unsafe {
            //TODO: handle error
            SetWindowPos(tile.window.id as HWND, std::ptr::null_mut(), x, y + *app_bar::HEIGHT.lock().unwrap(), width, height, 0);
        }
    }

    fn draw_tile(&self, tile: &Tile){
        let column_width = self.width / self.columns;
        let row_height = self.height / self.rows;

        let mut x = 0;
        
        if let Some(column) = tile.column {
            x = column_width * (column - 1);
        };

        let y = match tile.row {
            Some(row) => row_height * (row - 1),
            None => 0
        };

        let height = match tile.row {
            Some(_row) => row_height,
            None => self.height
        };  

        let mut width = match tile.column {
            Some(_column) => column_width,
            None => self.width
        };
        
        if let Some(rule) = &tile.window.rule {
            if rule.has_custom_titlebar {
                x = x - rule.x;
                width = width + rule.width;
            }
        }

        unsafe {
            //TODO: handle error
            SetWindowPos(tile.window.id as HWND, std::ptr::null_mut(), x, y + *app_bar::HEIGHT.lock().unwrap(), width, height, 0);
        }
    }

    pub fn print_grid(&self) -> () {
        if self.rows == 0 || self.columns == 0 {
            print!("\nEmpty\n\n");
            return;
        }

        let mut rows = [[std::ptr::null(); 10]; 10];

        for tile in &self.tiles {
            match tile.row {
                Some(row) => match tile.column {
                    Some(column) => rows[(row - 1) as usize][(column - 1) as usize] = &tile.window,
                    None => for i in 0..self.columns {
                        rows[(row - 1) as usize][i as usize] = &tile.window;
                    }
                },
                None => match tile.column {
                    Some(column) => for i in 0..self.rows {
                        rows[i as usize][(column - 1) as usize] = &tile.window;
                    }
                    None => rows[0][0] = &tile.window
                }
            }
            if CONFIG.remove_title_bar {
                self.draw_tile(tile);
            } else {
                self.draw_tile_with_title_bar(tile);
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
                            false => print!(" {}({}) |", window.title, window.id)
                        }
                    }
                }
            }
            print!("\n");
        }

        print!("\n");
    }
}