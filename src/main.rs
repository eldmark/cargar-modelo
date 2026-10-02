use raylib::prelude::*;

mod framebuffer;
mod line;
mod obj;
mod triangle;

use framebuffer::Framebuffer;
use triangle::triangle;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mesh = obj::load_obj("Untitled.obj")?;

    // Blender is Z-up: use x (right) and z (up) as the 2D position.
    let pts: Vec<(f32, f32)> = mesh.vertices.iter().map(|v| (v[0], v[2])).collect();

    // scale and translate so the model is centered and fully visible
    let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
    let min_y = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min);
    let max_y = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max);
    let scale = 0.9 * (WIDTH as f32 / (max_x - min_x)).min(HEIGHT as f32 / (max_y - min_y));
    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

    let screen: Vec<Vector2> = pts
        .iter()
        .map(|&(x, y)| {
            Vector2::new(
                WIDTH as f32 / 2.0 + (x - cx) * scale,
                HEIGHT as f32 / 2.0 - (y - cy) * scale, // screen y points down
            )
        })
        .collect();

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, Color::BLACK);
    framebuffer.clear();
    framebuffer.set_current_color(Color::WHITE);

    for f in &mesh.faces {
        triangle(&mut framebuffer, screen[f[0]], screen[f[1]], screen[f[2]]);
    }

    framebuffer.render_to_file("out.png");

    // show the framebuffer until the window is closed
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Modelo OBJ")
        .build();
    let texture = rl.load_texture_from_image(&thread, &framebuffer.color_buffer)?;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
    Ok(())
}
