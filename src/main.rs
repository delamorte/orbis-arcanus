use rltk::{Console, GameState, Rltk, RGB};
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;

rltk::add_wasm_support!();

const WIDTH: i32 = 40;
const HEIGHT: i32 = 25;

pub struct State {
    pub ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        
        player_input(self, ctx);
        self.run_systems();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        draw_map(&self.ecs, ctx);
        
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
        
    }
}


fn main() {

    let mut context = Rltk::init_raw(
        WIDTH as u32 * 32,
        HEIGHT as u32 * 48,
        "Orbis Arcanus",
    );


    let font = context.register_font(rltk::Font::load("resources/orbis_rust.png", (16, 24)));
/*     context.register_console(
        rltk::SparseConsole::init(WIDTH as u32, HEIGHT as u32, &context.backend), font,); */
    context.register_console_no_bg(
        rltk::SparseConsole::init(WIDTH as u32, HEIGHT as u32, &context.backend), font,);


    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    // Player entity
    gs.ecs
    .create_entity()
    .with(Position { x: player_x, y: player_y })
    .with(Renderable {
        glyph: 208,
        fg: RGB::named(rltk::YELLOW),
        bg: RGB::named(rltk::BLACK),
    })
    .with(Player{})
    .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
    .build();

    rltk::main_loop(context, gs);
}