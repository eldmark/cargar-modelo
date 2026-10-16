use raylib::prelude::*;

#[derive(Clone, Debug)]
pub struct Vertex {
    /// Position as loaded from the model.
    pub position: Vector3,
    /// RGB, each component in 0.0..=1.0.
    pub color: Vector3,
    /// Position after the vertex shader; the later stages read this one.
    pub transformed_position: Vector3,
}

impl Vertex {
    pub fn new(position: Vector3, color: Vector3) -> Self {
        Vertex {
            position,
            color,
            transformed_position: position,
        }
    }
}
