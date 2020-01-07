use rltk::{Console, GameState, Rltk, RGB, Point};
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
pub mod camera;
mod components;
pub use components::*;
mod gui;
mod gamelog;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai;
use monster_ai::MonsterAI;

rltk::add_wasm_support!();

const WIDTH: i32 = 40;
const HEIGHT: i32 = 20;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

pub struct State {
    pub ecs: World,
    pub runstate : RunState
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
    
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        camera::render_camera(&self.ecs, ctx);

        ctx.set_active_console(1);
        ctx.cls();
        gui::draw_ui(&self.ecs, ctx, WIDTH*2, HEIGHT*2);
        ctx.set_active_console(0);

        
    }
}


fn main() {

    let mut context = Rltk::init_raw(
        WIDTH as u32 * 32,
        HEIGHT as u32 * 48,
        "Orbis Arcanus",
    );
    context.with_post_scanlines(false);
    let font = context.register_font(rltk::Font::load("resources/orbis_tiles.png", (16, 24)));

    context.register_console_no_bg(
        rltk::SparseConsole::init(WIDTH as u32, HEIGHT as u32, &context.backend), font,);

    let ui_font = context.register_font(rltk::Font::load("resources/vga8x16.png", (8, 16)));
    let font_w = WIDTH*2;
    let font_h = HEIGHT*2;
    context.register_console(rltk::SparseConsole::init(font_w as u32, font_h as u32, &context.backend), ui_font);

    let mut gs = State {
        ecs: World::new(),
        runstate : RunState::Running
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Hidden>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    for room in map.rooms.iter().skip(1) {
        let (x,y) = room.center();
        gs.ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{
                glyph: 196,
                fg: RGB::named(rltk::PINK),
                bg: RGB::named(rltk::BLACK),
                render_order: 0
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .build();
    }
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Orbis Arcanus...".to_string()] });

    // Player entity
    gs.ecs
    .create_entity()
    .with(Position { x: player_x, y: player_y })
    .with(Renderable {
        glyph: 208,
        fg: RGB::named(rltk::YELLOW),
        bg: RGB::named(rltk::BLACK),
        render_order: 0
    })
    .with(Player{})
    .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
    .build();

    rltk::main_loop(context, gs);
}