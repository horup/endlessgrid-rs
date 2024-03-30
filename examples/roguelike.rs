use std::collections::HashMap;

use egrid::*;
use macroquad::prelude::*;
use tiled::Loader;
#[derive(Default, Clone)]
struct Tile {
    pub blocks_los: bool,
}


fn load_map(grid:&mut EGrid<Tile>) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("examples/roguelike.tmx").unwrap();
    for layer in map.layers() {
        let Some(layer) = layer.as_tile_layer() else {
            continue;
        };
        let tiled::TileLayer::Infinite(layer) = layer else {
            continue;
        };
        for (chunk_pos, chunk) in layer.chunks() {
            for x in 0..tiled::ChunkData::WIDTH as i32 {
                for y in 0..tiled::ChunkData::HEIGHT as i32 {
                    if let Some(tile) = chunk.get_tile(x, y) {
                        let tile_pos = (
                            chunk_pos.0 * tiled::ChunkData::WIDTH as i32 + x,
                            chunk_pos.1 * tiled::ChunkData::HEIGHT as i32 + y,
                        );
                        let classes = tile.get_tile().unwrap().user_type.clone().unwrap_or_default();
                        let classes = classes.split(" ").map(|x|(x.to_owned(), ()));
                        let classes:HashMap<String,()> = classes.collect();
                        grid.insert(tile_pos, Tile {
                            blocks_los:classes.contains_key("solid")
                        });
                    }
                }
            }
        }
    }
}

#[macroquad::main("Roguelike")]
async fn main() {
    let mut grid = EGrid::default() as EGrid<Tile>;
    load_map(&mut grid);
    loop {
        let zoom = 2.0;
        let camera = Camera2D {
            zoom:Vec2::new(zoom / screen_width(), zoom / screen_height()),
            ..Default::default()
        };
        set_camera(&camera);
        clear_background(BLACK);
        let view_distance = 32;

        let player_pos = (16, 16);

        let tile_size_px = 16.0;
        for y in (player_pos.1 - view_distance)..(player_pos.1 + view_distance) {
            for x in (player_pos.0 - view_distance)..(player_pos.0 + view_distance) {
                if let Some(tile) = grid.get((x, y)) {
                    let color = if tile.blocks_los { WHITE } else { GRAY };
                    draw_rectangle(
                        x as f32 * tile_size_px,
                        y as f32 * tile_size_px,
                        tile_size_px as f32,
                        tile_size_px as f32,
                        color,
                    );
                }
            }
        }

        // draw player
        draw_circle(
            player_pos.0 as f32 * tile_size_px as f32 + tile_size_px as f32 / 2.0,
            player_pos.1 as f32 * tile_size_px as f32 + tile_size_px as f32 / 2.0,
            tile_size_px as f32 / 2.0,
            WHITE,
        );

        /*draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
                draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
                draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

                draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);
        */
        next_frame().await
    }
}
