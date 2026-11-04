use crate::vertex::Vertex;

pub fn primitive_assembly(transformed_vertices: &[Vertex]) -> Vec<[Vertex; 3]> {
    assert!(
        transformed_vertices.len() % 3 == 0,
        "La cantidad de vértices debe ser múltiplo de 3",
    );

    transformed_vertices
        .chunks_exact(3)
        .map(|group| [group[0].clone(), group[1].clone(), group[2].clone()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::*;

    fn numbered(n: usize) -> Vec<Vertex> {
        (0..n)
            .map(|i| Vertex::new(Vector3::new(i as f32, 0.0, 0.0), Vector3::zero()))
            .collect()
    }

    #[test]
    fn six_vertices_make_two_triangles_in_order() {
        let tris = primitive_assembly(&numbered(6));
        let ids: Vec<[f32; 3]> = tris
            .iter()
            .map(|t| [t[0].position.x, t[1].position.x, t[2].position.x])
            .collect();
        assert_eq!(ids, vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
    }

    #[test]
    #[should_panic(expected = "múltiplo de 3")]
    fn seven_vertices_panic() {
        primitive_assembly(&numbered(7));
    }
}
