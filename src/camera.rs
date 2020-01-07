use specs::prelude::*;
use super::{Map,TileType,Position,Renderable,Hidden};
use rltk::{Point, Rltk, Console, RGB};

const SHOW_BOUNDARIES : bool = false;

pub fn get_screen_bounds(ecs: &World, ctx : &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    //let (x_chars, y_chars) = ctx.get_char_size();
    let (x_chars, y_chars) = (34, 15);

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

pub fn render_camera(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    let map_width = map.width-1;
    let map_height = map.height-1;

    let mut y = 0;
    for ty in min_y .. max_y {
        let mut x = 0;
        for tx in min_x .. max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(rltk::GRAY), RGB::named(rltk::BLACK), 109);                
            }
            x += 1;
        }
        y += 1;
    }

    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let map = ecs.fetch::<Map>();

    let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
    for (pos, render, _hidden) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] { 
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0 && entity_screen_x < map_width && entity_screen_y > 0 && entity_screen_y < map_height {
                ctx.set(entity_screen_x, entity_screen_y, render.fg, render.bg, render.glyph);
            }
        }
    }

}

fn get_tile_glyph(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = 239;
            fg = RGB::from_u8(120, 80, 40);
        }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_u8(120, 50, 40);
        }
/*         TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        } */
    }
    // if map.bloodstains.contains(&idx) { bg = RGB::from_f32(0.75, 0., 0.); }
    if !map.visible_tiles[idx] { 
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // Don't show stains out of visual range
    }

    (glyph, fg, bg)
}

fn wall_glyph(map : &Map, x: i32, y:i32) -> u8 {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
    if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
    if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
    if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 246 } // Pillar because we can't see neighbors
        1 => { 246 } // Wall only to the north
        2 => { 224 } // Wall only to the south
        3 => { 224 } // Wall to the north and south
        4 => { 246 } // Wall only to the west
        5 => { 246 } // Wall to the north and west
        6 => { 224 } // Wall to the south and west
        7 => { 224 } // Wall to the north, south and west
        8 => { 246 } // Wall only to the east
        9 => { 246 } // Wall to the north and east
        10 => { 224 } // Wall to the south and east
        11 => { 224 } // Wall to the north, south and east
        12 => { 246 } // Wall to the east and west
        13 => { 246 } // Wall to the east, west, and north
        14 => { 224 } // Wall to the east, west, and south
        _ => { 224 } // We missed one?
    }
}
fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}