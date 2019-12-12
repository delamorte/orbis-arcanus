use rltk::{ RGB, Rltk, Console };
use specs::prelude::*;

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_ui(ecs: &World, ctx : &mut Rltk, w : i32, h : i32) {
    use rltk::to_cp437;
    let box_color : RGB = RGB::from_u8(70, 60, 60);
    let black = RGB::named(rltk::BLACK);

    draw_hollow_box(ctx, 0, 0, w-1, h-1, box_color, black); // Overall box
    draw_hollow_box(ctx, 6, 0, w-7, h-5, box_color, black); // Map box
    draw_hollow_box(ctx, 0, h-5, w-1, 4, box_color, black); // Log box
    draw_hollow_box(ctx, 0, 0, 6, 4, box_color, black); // Top-left panel

    ctx.set(0, h-5, box_color, black, to_cp437('├'));
    ctx.set(0, 4, box_color, black, to_cp437('├'));
    ctx.set(6, 0, box_color, black, to_cp437('┬'));
    ctx.set(6, h-5, box_color, black, to_cp437('┴'));
    ctx.set(6, 4, box_color, black, to_cp437('┤'));
    ctx.set(w-1, h-5, box_color, black, to_cp437('┤'));
}
