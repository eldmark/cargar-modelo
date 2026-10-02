use crate::framebuffer::Framebuffer;
use crate::line::line;
use raylib::prelude::*;

/// Triangle outline: three lines.
pub fn triangle(framebuffer: &mut Framebuffer, a: Vector2, b: Vector2, c: Vector2) {
    line(framebuffer, a, b);
    line(framebuffer, b, c);
    line(framebuffer, c, a);
}
