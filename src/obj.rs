use std::fs;

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
}

pub fn load_obj(path: &str) -> Result<Mesh, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let c: Vec<f32> = it.take(3).map(str::parse).collect::<Result<_, _>>()?;
                vertices.push([c[0], c[1], c[2]]);
            }
            Some("f") => {
                // "v/vt/vn": only the position index matters; OBJ indices are 1-based
                let idx: Vec<usize> = it
                    .map(|s| s.split('/').next().unwrap().parse::<usize>().map(|i| i - 1))
                    .collect::<Result<_, _>>()?;
                for i in 1..idx.len() - 1 {
                    faces.push([idx[0], idx[i], idx[i + 1]]);
                }
            }
            _ => {}
        }
    }
    Ok(Mesh { vertices, faces })
}
