use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(
            self.width as i32,
            self.height as i32,
            self.background_color,
        );
    }

    /// Writes `color` (RGB in 0.0..=1.0) at (x, y); positions outside the image are ignored.
    pub fn point(&mut self, x: i32, y: i32, color: Vector3) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let to_u8 = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
            let color = Color::new(to_u8(color.x), to_u8(color.y), to_u8(color.z), 255);
            self.color_buffer.draw_pixel(x, y, color);
        }
    }

    pub fn render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }
}
