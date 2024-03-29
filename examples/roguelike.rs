use macroquad::prelude::*;
use egrid::*;

#[derive(Default, Clone)]
struct Tile {
    pub blocks:bool
}
#[macroquad::main("Roguelike")]
async fn main() {
    let mut grid = EGrid::default() as EGrid<Tile>;
    let size = 32;
    for y in 0..size {
        for x in 0..size {
            grid.insert([x,y], Tile {
                blocks:if y == 8 { true } else { false }
            });
        }
    }
    loop {
        clear_background(BLACK);
        let view_distance = 16;

        let player_pos = (16, 16);


        let tile_size_px = 16.0;
        for y in (player_pos.1 - view_distance)..(player_pos.1 + view_distance) {
            for x in (player_pos.0 - view_distance)..(player_pos.0 + view_distance) {
                if let Some(tile) = grid.get((x,y)) {
                    let color = if tile.blocks { WHITE} else { GRAY };
                    draw_rectangle(x as f32 * tile_size_px, y as f32 * tile_size_px, tile_size_px as f32, tile_size_px as f32, color);

                }
            }
        }


        // draw player
        draw_circle(player_pos.0 as f32 * tile_size_px as f32 + tile_size_px as f32 /2.0, player_pos.1 as f32  * tile_size_px as f32 + tile_size_px as f32 / 2.0, tile_size_px as f32 / 2.0, WHITE);


        /*draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);
*/
        next_frame().await
    }
}