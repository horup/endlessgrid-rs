use std::{collections::HashMap, f32::consts::PI, time::Instant};

use endlessgrid::*;
use ::glam::Vec2;
use macroquad::{prelude::*};
use slotmap::{DefaultKey, SlotMap};
use tiled::Loader;
#[derive(Default, Clone)]
struct Tile {
    pub index:u16,
    pub solid: bool,
    pub entities:HashMap<DefaultKey, ()>
}

#[derive(Default, Clone)]
pub struct Entity {
    pub pos:IVec2,
    pub index:u16,
    pub is_player:bool
}


fn load_map(grid:&mut Grid<Tile>, entities:&mut SlotMap<DefaultKey, Entity>) {
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
                        let mut keys = HashMap::default();
                        if classes.contains_key("player") {
                            let key = entities.insert(Entity {
                                index:tile.id() as u16,
                                pos:tile_pos.into(),
                                is_player:true
                            });
                            keys.insert(key, ());
                        }
                         
                        grid.insert(tile_pos, Tile {
                            index:if classes.contains_key("entity") { 0 } else { tile.id() as u16 },
                            solid:classes.contains_key("solid"),
                            entities:keys
                        });
                    }
                }
            }
        }
    }
}

fn draw_atlas(texture:&Texture2D, x:f32, y:f32, index:f32, color:Color, tile_size:f32) {
    let nx = texture.width() / tile_size;
    let ny = texture.height() / tile_size;
    let index = index as u16;

    let sx = index % nx as u16;
    let sy = index / ny as u16;
    let sx = sx as f32 * tile_size;
    let sy = sy as f32 * tile_size;
    draw_texture_ex(texture, x, y, color, DrawTextureParams {
        dest_size:Some((tile_size, tile_size).into()),
        source:Some(Rect::new(sx, sy, tile_size, tile_size)),
        ..Default::default()
    });
}

fn move_entity(entity:DefaultKey, to:(i32, i32), grid:&mut Grid<Tile>, entities:&mut SlotMap<DefaultKey, Entity>) {
    let entity_key = entity;
    let Some(entity) = entities.get_mut(entity_key) else { return };
    let Some(to_tile) = grid.get_mut(to) else { return };

    if to_tile.solid { return };
    let org_pos = entity.pos;
    to_tile.entities.insert(entity_key, ());
    let Some(from_tile) = grid.get_mut((org_pos.x, org_pos.y)) else { return };
    from_tile.entities.remove(&entity_key);
    entity.pos = to.into();

}

fn input(player_entity:DefaultKey, grid:&mut Grid<Tile>, entities:&mut SlotMap<DefaultKey, Entity>, instant:&mut Instant) {
    if (Instant::now() - *instant).as_millis() < 100 {
        return;
    }
    
    let Some(entity) = entities.get(player_entity) else { return };
    let mut pos = entity.pos;
    if is_key_down(KeyCode::W) {
        pos.y -= 1;
    }
    else if is_key_down(KeyCode::S) {
        pos.y += 1;
    }
    else if is_key_down(KeyCode::A) {
        pos.x -= 1;
    }
    else if is_key_down(KeyCode::D) {
        pos.x += 1;
    }

    if pos != entity.pos {
        *instant = Instant::now();
        move_entity(player_entity, (pos.x, pos.y), grid, entities);
    }
}

#[macroquad::main("Roguelike")]
async fn main() {
    let mut entities = SlotMap::default();
    let mut grid = Grid::default() as Grid<Tile>;
    load_map(&mut grid, &mut entities);
    let player_entity = entities.iter().filter(|x|x.1.is_player).next().expect("player not ofund").0;
    let tile_size_px = 8.0;
    let tilemap_texture = load_texture("examples/tileset.png").await.unwrap();
    tilemap_texture.set_filter(FilterMode::Nearest);
    let mut instant = Instant::now();
    loop {
        input(player_entity, &mut grid, &mut entities, &mut instant);
        let zoom = 4.0;
       
        clear_background(WHITE);
        let view_distance = 16.0;

        let player_pos = entities.get(player_entity).unwrap().pos;
        let player_pos = (player_pos.x, player_pos.y);

        let camera = Camera2D {
            target:macroquad::prelude::Vec2::new(player_pos.0 as f32 * tile_size_px, player_pos.1 as f32 * tile_size_px),
            zoom:macroquad::prelude::Vec2::new(zoom / screen_width(), zoom / screen_height()),
            ..Default::default()
        };
        set_camera(&camera);

        let mut tiles = HashMap::new();
        let r = 512;
        for a in 0..r {
            let a =  a as f32 / r as f32 * PI * 2.0;
            let start = Vec2::new(player_pos.0 as f32 + 0.5, player_pos.1 as f32 + 0.5);
            let v = Vec2::new(a.cos(), a.sin());
            let end = v * view_distance + start;

            grid.cast_ray(start, end, |x|{
                tiles.insert(x.index, ());
                if x.cell.solid {
                    return true;
                }

                return false;
            });
        }

        for (x,y) in tiles.keys() {
            let Some(tile) = grid.get((*x,*y)) else {continue;};
            let x = *x as f32 * tile_size_px;
            let y = *y as f32 * tile_size_px;
            draw_atlas(&tilemap_texture, x, y, tile.index as f32, WHITE, tile_size_px);
            for entity in tile.entities.keys() {
                let Some(entity) = entities.get(*entity) else { continue;};
                draw_atlas(&tilemap_texture, x, y, entity.index as f32, WHITE, tile_size_px);
            }
        }
        /*for y in (player_pos.1 - view_distance)..(player_pos.1 + view_distance) {
            for x in (player_pos.0 - view_distance)..(player_pos.0 + view_distance) {
                if let Some(tile) = grid.get((x, y)) {
                    let x = x as f32 * tile_size_px;
                    let y = y as f32 * tile_size_px;
                    draw_atlas(&tilemap_texture, x, y, tile.index as f32, WHITE, tile_size_px);
                    for entity in tile.entities.keys() {
                        let Some(entity) = entities.get(*entity) else { continue;};
                        draw_atlas(&tilemap_texture, x, y, entity.index as f32, WHITE, tile_size_px);
                    }
                }
            }
        }*/
        next_frame().await
    }
}
