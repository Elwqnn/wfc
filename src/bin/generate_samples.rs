//! Generate sample pattern images for WFC

use wfc::{Color, Sample};

fn main() {
    let samples_dir = std::path::Path::new("samples");
    std::fs::create_dir_all(samples_dir).unwrap();

    create_pipes(samples_dir);

    create_maze(samples_dir);

    create_circuits(samples_dir);

    create_flowers(samples_dir);

    create_knots(samples_dir);

    create_stripes(samples_dir);

    println!("Generated samples in ./samples/");
}

fn create_pipes(dir: &std::path::Path) {
    let bg: Color = [32, 32, 48];
    let pipe: Color = [64, 128, 192];
    let joint: Color = [96, 192, 255];

    #[rustfmt::skip]
    let pixels = vec![
        bg, bg, bg, bg, bg, bg, bg, bg,
        bg, joint, pipe, pipe, pipe, joint, bg, bg,
        bg, pipe, bg, bg, bg, pipe, bg, bg,
        bg, pipe, bg, joint, pipe, joint, pipe, bg,
        bg, pipe, bg, pipe, bg, bg, pipe, bg,
        bg, joint, pipe, joint, bg, joint, pipe, bg,
        bg, bg, bg, pipe, bg, pipe, bg, bg,
        bg, bg, bg, joint, pipe, joint, bg, bg,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("pipes.png")).unwrap();
}

fn create_maze(dir: &std::path::Path) {
    let wall: Color = [40, 40, 60];
    let path: Color = [200, 180, 140];

    #[rustfmt::skip]
    let pixels = vec![
        wall, wall, wall, wall, wall, wall, wall, wall,
        wall, path, path, path, wall, path, path, wall,
        wall, path, wall, path, wall, path, wall, wall,
        wall, path, wall, path, path, path, path, wall,
        wall, path, wall, wall, wall, wall, path, wall,
        wall, path, path, path, path, wall, path, wall,
        wall, wall, wall, wall, path, path, path, wall,
        wall, wall, wall, wall, wall, wall, wall, wall,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("maze.png")).unwrap();
}

fn create_circuits(dir: &std::path::Path) {
    let bg: Color = [20, 30, 20];
    let trace: Color = [50, 200, 50];
    let node: Color = [200, 200, 50];

    #[rustfmt::skip]
    let pixels = vec![
        bg, bg, trace, bg, bg, bg, trace, bg,
        bg, node, trace, trace, trace, node, trace, bg,
        trace, trace, bg, bg, bg, trace, bg, bg,
        bg, trace, bg, node, trace, trace, trace, trace,
        bg, trace, bg, trace, bg, bg, bg, bg,
        trace, trace, trace, trace, bg, node, trace, bg,
        bg, bg, bg, bg, bg, trace, bg, bg,
        bg, bg, trace, trace, trace, trace, bg, bg,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("circuits.png")).unwrap();
}

fn create_flowers(dir: &std::path::Path) {
    let grass: Color = [60, 140, 60];
    let stem: Color = [40, 100, 40];
    let petal: Color = [255, 100, 150];
    let center: Color = [255, 220, 50];

    #[rustfmt::skip]
    let pixels = vec![
        grass, grass, petal, petal, petal, grass, grass, grass,
        grass, petal, petal, center, petal, petal, grass, grass,
        grass, petal, center, center, center, petal, grass, grass,
        grass, petal, petal, center, petal, petal, grass, grass,
        grass, grass, petal, stem, petal, grass, grass, grass,
        grass, grass, grass, stem, grass, grass, grass, grass,
        grass, grass, grass, stem, grass, grass, petal, grass,
        grass, grass, grass, stem, grass, petal, center, petal,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("flowers.png")).unwrap();
}

fn create_knots(dir: &std::path::Path) {
    let bg: Color = [240, 230, 210];
    let rope: Color = [139, 90, 43];
    let shadow: Color = [100, 60, 30];

    #[rustfmt::skip]
    let pixels = vec![
        bg, bg, rope, rope, bg, bg, bg, bg,
        bg, rope, shadow, rope, rope, bg, bg, bg,
        rope, shadow, bg, bg, rope, rope, bg, bg,
        rope, bg, bg, bg, bg, rope, rope, bg,
        rope, rope, bg, bg, bg, bg, rope, rope,
        bg, rope, rope, bg, bg, rope, shadow, rope,
        bg, bg, rope, rope, rope, shadow, bg, bg,
        bg, bg, bg, rope, rope, bg, bg, bg,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("knots.png")).unwrap();
}

fn create_stripes(dir: &std::path::Path) {
    let c1: Color = [65, 105, 225]; // Royal blue
    let c2: Color = [255, 255, 255]; // White
    let c3: Color = [220, 20, 60]; // Crimson

    #[rustfmt::skip]
    let pixels = vec![
        c1, c1, c2, c2, c3, c3, c2, c2,
        c1, c1, c2, c2, c3, c3, c2, c2,
        c2, c2, c3, c3, c2, c2, c1, c1,
        c2, c2, c3, c3, c2, c2, c1, c1,
        c3, c3, c2, c2, c1, c1, c2, c2,
        c3, c3, c2, c2, c1, c1, c2, c2,
        c2, c2, c1, c1, c2, c2, c3, c3,
        c2, c2, c1, c1, c2, c2, c3, c3,
    ];

    let sample = Sample::new(8, 8, pixels);
    sample.save(&dir.join("stripes.png")).unwrap();
}
