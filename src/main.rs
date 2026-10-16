use raylib::prelude::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

mod fragment;
mod framebuffer;
mod line;
mod obj;
mod primitive;
mod rasterizer;
mod shaders;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;
use primitive::primitive_assembly;
use rasterizer::{TriangleMode, rasterize};
use shaders::{fragment_shader, vertex_shader};
use vertex::Vertex;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Data shared by every vertex and fragment of one draw call.
pub struct Uniforms {
    pub model_matrix: Matrix,
    /// How the rasterizer turns each triangle into fragments.
    pub triangle_mode: TriangleMode,
}

const MODES: [(TriangleMode, &str); 3] = [
    (TriangleMode::Wireframe, "wireframe"),
    (TriangleMode::FilledBarycentric, "barycentric"),
    (TriangleMode::FilledScanline, "scanline"),
];

type Rows = [[f32; 4]; 4];

fn multiply(a: &Rows, b: &Rows) -> Rows {
    let mut out = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            out[r][c] = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

/// Model matrix = translation * scale * rotation (rotation is applied first,
/// around x, then y, then z; angles in radians).
fn create_model_matrix(translation: Vector3, scale: f32, rotation: Vector3) -> Matrix {
    let (sx, cx) = rotation.x.sin_cos();
    let (sy, cy) = rotation.y.sin_cos();
    let (sz, cz) = rotation.z.sin_cos();
    let rot_x = [[1.0, 0.0, 0.0, 0.0], [0.0, cx, -sx, 0.0], [0.0, sx, cx, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let rot_y = [[cy, 0.0, sy, 0.0], [0.0, 1.0, 0.0, 0.0], [-sy, 0.0, cy, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let rot_z = [[cz, -sz, 0.0, 0.0], [sz, cz, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let scale_m = [[scale, 0.0, 0.0, 0.0], [0.0, scale, 0.0, 0.0], [0.0, 0.0, scale, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let translate_m = [
        [1.0, 0.0, 0.0, translation.x],
        [0.0, 1.0, 0.0, translation.y],
        [0.0, 0.0, 1.0, translation.z],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let rotation_m = multiply(&rot_z, &multiply(&rot_y, &rot_x));
    let m = multiply(&translate_m, &multiply(&scale_m, &rotation_m));

    Matrix {
        m0: m[0][0], m4: m[0][1], m8: m[0][2], m12: m[0][3],
        m1: m[1][0], m5: m[1][1], m9: m[1][2], m13: m[1][3],
        m2: m[2][0], m6: m[2][1], m10: m[2][2], m14: m[2][3],
        m3: m[3][0], m7: m[3][1], m11: m[3][2], m15: m[3][3],
    }
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    // 1. Procesamiento de vértices
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización
    let fragments = rasterize(&triangles, uniforms.triangle_mode);

    // 4. Procesamiento de fragmentos
    for fragment in fragments {
        let color = fragment_shader(&fragment);

        framebuffer.point(fragment.position.x as i32, fragment.position.y as i32, color);
    }
}

/// One vertex per triangle corner, three per face, in face order.
fn load_vertex_array(path: &str) -> Result<Vec<Vertex>, Box<dyn std::error::Error>> {
    let mesh = obj::load_obj(path)?;
    let color = Vector3::new(1.0, 0.6, 0.2);
    Ok(mesh
        .faces
        .iter()
        .flat_map(|f| f.iter().map(|&i| mesh.vertices[i]))
        .map(|[x, y, z]| Vertex::new(Vector3::new(x, y, z), color))
        .collect())
}

/// Centers the model and scales it to 90 % of the window. Blender is Z-up, so
/// a 90° turn around x puts the model's z on the screen's y axis (pointing down).
fn fit_to_window(vertex_array: &[Vertex]) -> Matrix {
    let xs = vertex_array.iter().map(|v| v.position.x);
    let zs = vertex_array.iter().map(|v| v.position.z);
    let (min_x, max_x) = xs.fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
    let (min_z, max_z) = zs.fold((f32::MAX, f32::MIN), |(lo, hi), z| (lo.min(z), hi.max(z)));
    let scale = 0.9 * (WIDTH as f32 / (max_x - min_x)).min(HEIGHT as f32 / (max_z - min_z));
    let (cx, cz) = ((min_x + max_x) / 2.0, (min_z + max_z) / 2.0);

    create_model_matrix(
        Vector3::new(WIDTH as f32 / 2.0 - cx * scale, HEIGHT as f32 / 2.0 + cz * scale, 0.0),
        scale,
        Vector3::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
    )
}

fn median(mut times: Vec<Duration>) -> Duration {
    times.sort();
    times[times.len() / 2]
}

/// Times only fragment generation (rasterize) for each mode; no window is opened.
fn benchmark_triangles(vertex_array: &[Vertex], model_matrix: Matrix) {
    const RUNS: usize = 100;
    const REPEATS: usize = 9;

    let uniforms = Uniforms { model_matrix, triangle_mode: TriangleMode::Wireframe };
    let transformed: Vec<Vertex> = vertex_array.iter().map(|v| vertex_shader(v, &uniforms)).collect();
    let triangles = primitive_assembly(&transformed);
    println!("{} triángulos, {RUNS} ejecuciones por medición, mediana de {REPEATS} mediciones\n", triangles.len());

    let mut pixels = Vec::new();
    for (mode, name) in MODES {
        let mut times = Vec::new();
        for _ in 0..REPEATS {
            let start = Instant::now();
            for _ in 0..RUNS {
                let fragments = rasterize(black_box(&triangles), mode);
                black_box(fragments);
            }
            times.push(start.elapsed());
        }
        let fragments = rasterize(&triangles, mode);
        let (lo, hi) = (*times.iter().min().unwrap(), *times.iter().max().unwrap());
        let med = median(times);
        println!(
            "{name:<12} fragmentos: {:>7}  mediana: {med:>10.3?}  ({:.3?} por ejecución, min {lo:.3?}, max {hi:.3?})",
            fragments.len(),
            med / RUNS as u32,
        );

        let mut positions: Vec<(i32, i32)> = fragments
            .iter()
            .map(|f| (f.position.x as i32, f.position.y as i32))
            .collect();
        positions.sort();
        pixels.push(positions);
    }
    println!(
        "\n¿Baricéntrico y scanline generan las mismas posiciones? {}",
        if pixels[1] == pixels[2] { "sí" } else { "no" }
    );
    // walk both sorted lists to find the positions only one of them produced
    let (bary, scan) = (&pixels[1], &pixels[2]);
    let (mut i, mut j) = (0, 0);
    while i < bary.len() || j < scan.len() {
        if j == scan.len() || (i < bary.len() && bary[i] < scan[j]) {
            println!("  solo en baricéntrico: {:?}", bary[i]);
            i += 1;
        } else if i == bary.len() || scan[j] < bary[i] {
            println!("  solo en scanline: {:?}", scan[j]);
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vertex_array = load_vertex_array("Untitled.obj")?;
    let model_matrix = fit_to_window(&vertex_array);

    if std::env::args().any(|a| a == "--benchmark-triangles") {
        benchmark_triangles(&vertex_array, model_matrix);
        return Ok(());
    }

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, Color::BLACK);
    let mut uniforms = Uniforms { model_matrix, triangle_mode: TriangleMode::Wireframe };

    for (mode, name) in MODES {
        uniforms.triangle_mode = mode;
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array);
        framebuffer.render_to_file(&format!("out_{name}.png"));
    }

    // show the framebuffer until the window is closed; keys 1-3 switch the mode
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Modelo OBJ — 1: wireframe, 2: baricéntrico, 3: scanline")
        .build();

    let mut current = 0;
    let mut dirty = true;
    let mut texture = rl.load_texture_from_image(&thread, &framebuffer.color_buffer)?;

    while !rl.window_should_close() {
        for (i, key) in [KeyboardKey::KEY_ONE, KeyboardKey::KEY_TWO, KeyboardKey::KEY_THREE].into_iter().enumerate() {
            if rl.is_key_pressed(key) && i != current {
                current = i;
                dirty = true;
            }
        }
        if dirty {
            uniforms.triangle_mode = MODES[current].0;
            framebuffer.clear();
            render(&mut framebuffer, &uniforms, &vertex_array);
            texture = rl.load_texture_from_image(&thread, &framebuffer.color_buffer)?;
            dirty = false;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text(MODES[current].1, 10, 10, 20, Color::GRAY);
    }
    Ok(())
}
