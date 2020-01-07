use rltk::{ RGB, Rltk, Console, RandomNumberGenerator, BaseMap, Algorithm2D, Point };
use super::{Rect};
use std::cmp::{max, min};
use specs::prelude::*;

const MAPWIDTH : usize = 50;
const MAPHEIGHT : usize = 50;
const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

#[derive(Default)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
}

impl Map {

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room : &Rect) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }


    /// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
    /// look awful.
    pub fn new_map_test() -> Map {
        let mut map = Map{
            tiles : vec![TileType::Wall; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles : vec![false; MAPCOUNT],
            visible_tiles : vec![false; MAPCOUNT]
        };
        // Make the boundaries walls
        let boundaries = Rect::new(0, 0, map.width-2, map.height-2);
        map.apply_room_to_map(&boundaries);
        map.rooms.push(boundaries);
        // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
        // First, obtain the thread-local RNG:
        let mut rng = rltk::RandomNumberGenerator::new();

        for _i in 0..400 {
            let x = rng.roll_dice(1, map.width-2);
            let y = rng.roll_dice(1, map.height-2);
            let idx = map.xy_idx(x, y);
            if idx != map.xy_idx(map.width/2, map.height/2) {
                map.tiles[idx] = TileType::Wall;
            }
        }

        map
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map{
            tiles : vec![TileType::Wall; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles : vec![false; MAPCOUNT],
            visible_tiles : vec![false; MAPCOUNT]
        };

        const MAX_ROOMS : i32 = 15;
        const MIN_SIZE : i32 = 3;
        const MAX_SIZE : i32 = 6;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.range(7, map.width - w - 2);
            let y = rng.range(1, map.height - h - 2);
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:i32) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, _idx:i32) -> Vec<(i32, f32)> {
        Vec::new()
    }

    fn get_pathing_distance(&self, idx1:i32, idx2:i32) -> f32 {
        let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn point2d_to_index(&self, pt: Point) -> i32 {
        (pt.y * self.width) + pt.x
    }

    fn index_to_point2d(&self, idx:i32) -> Point {
        Point{ x: idx % self.width, y: idx / self.width }
    }
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

pub fn draw_map(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;

    for (idx,tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = 239;
                    fg = RGB::from_u8(120, 80, 40);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_u8(120, 50, 40);
                }
            }
            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        // Move the coordinates
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}