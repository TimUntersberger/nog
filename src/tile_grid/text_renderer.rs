use crate::tile_grid::tile_render_info::TileRenderInfo;

pub struct TextRenderer { }

impl TextRenderer {
    pub fn render(width: u32, height: u32, render_infos: Vec::<TileRenderInfo>) -> String {
        let mut buffer = vec![vec![" ".to_string(); height as usize]; width as usize];

        let mut letter_counter = 65;
        let mut tile_legend = String::new();
        for render_info in render_infos {
            let letter = std::char::from_u32(letter_counter).unwrap().to_string();
            tile_legend = tile_legend + &format!("{} ~ NodeID:{} WinID:{} - {} Size: {} Order: {}\n", letter,
                                        render_info.debug_id, 
                                        render_info.window.id, 
                                        render_info.window.get_title().unwrap_or("".to_string()).clone(),
                                        render_info.debug_size, 
                                        render_info.debug_order);
            buffer = TextRenderer::add_to_buffer(buffer, render_info, letter);
            letter_counter += 1;
            if letter_counter > 90 {
                letter_counter = 65;
            }
        }

        let mut result = "\n".to_string();
        for current_y in 0..height as usize {
            for current_x in 0..width as usize {
                result += &buffer[current_x][current_y].to_string();
            }
            result += "\n";
        }
        result += &tile_legend;

        result
    }

    fn add_to_buffer(mut buffer: Vec::<Vec::<String>>, 
                     render_info: TileRenderInfo, letter: String) -> Vec::<Vec::<String>> {
        let (min_x, max_x, min_y, max_y) = (render_info.x as usize, (render_info.x + render_info.width) as usize,
                                            render_info.y as usize, (render_info.y + render_info.height) as usize);

        for local_x in min_x..max_x {
            for local_y in min_y..max_y {
                buffer[local_x][local_y] = 
                    if local_x == min_x || local_x == max_x - 1 || local_y == min_y || local_y == max_y - 1 {
                        "█".to_string()
                    } else {
                        letter.to_string()
                    }
            }
        }

        buffer
    }
}
