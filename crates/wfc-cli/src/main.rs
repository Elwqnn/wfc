use std::path::{Path, PathBuf};
use std::process;

use clap::{Args, Parser, Subcommand, ValueEnum};
use wfc_core::{Boundary, Color, Config, RunOutcome, Sample, Wfc, default_pipe_sample};

#[derive(Clone, ValueEnum)]
enum BoundaryArg {
    /// No wrapping — hard edges
    Fixed,
    /// Wrap horizontally (left/right connect)
    PeriodicX,
    /// Wrap vertically (top/bottom connect)
    PeriodicY,
    /// Wrap both axes (toroidal)
    Periodic,
}

impl From<BoundaryArg> for Boundary {
    fn from(b: BoundaryArg) -> Self {
        match b {
            BoundaryArg::Fixed => Boundary::Fixed,
            BoundaryArg::PeriodicX => Boundary::PeriodicX,
            BoundaryArg::PeriodicY => Boundary::PeriodicY,
            BoundaryArg::Periodic => Boundary::Periodic,
        }
    }
}

/// Wave Function Collapse image generator
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
struct RunArgs {
    /// Input sample image (default: built-in pipes)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output image path
    #[arg(short, long, default_value = "output.png")]
    output: PathBuf,

    /// Output width in cells
    #[arg(short = 'W', long, default_value_t = 32)]
    width: usize,

    /// Output height in cells
    #[arg(short = 'H', long, default_value_t = 32)]
    height: usize,

    /// Pattern size NxN
    #[arg(short, long, default_value_t = 3)]
    pattern_size: usize,

    /// RNG seed for deterministic output
    #[arg(short, long)]
    seed: Option<u64>,

    /// Disable symmetry (rotations/reflections)
    #[arg(long)]
    no_symmetry: bool,

    /// Output boundary mode
    #[arg(long, value_enum, default_value_t = BoundaryArg::Fixed)]
    boundary: BoundaryArg,

    /// Max retries on contradiction
    #[arg(short, long, default_value_t = 10)]
    retries: usize,
}

#[derive(Subcommand)]
enum Command {
    /// Run WFC to generate an output image
    Run(RunArgs),
    /// Generate built-in sample pattern images into a directory
    GenerateSamples {
        /// Output directory
        #[arg(default_value = "samples")]
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => cmd_run(args),
        Command::GenerateSamples { dir } => cmd_generate_samples(&dir),
    }
}

fn cmd_run(args: RunArgs) {
    let RunArgs {
        input,
        output,
        width,
        height,
        pattern_size,
        seed,
        no_symmetry,
        boundary,
        retries,
    } = args;
    let sample = match &input {
        Some(path) => Sample::from_image(path).unwrap_or_else(|e| {
            eprintln!("Error loading sample '{}': {}", path.display(), e);
            process::exit(1);
        }),
        None => default_pipe_sample(),
    };

    let config = Config {
        pattern_size,
        output_width: width,
        output_height: height,
        periodic_input: true,
        boundary: boundary.into(),
        symmetry: !no_symmetry,
        ground: false,
        sides: false,
        seed,
        ..Default::default()
    };

    for attempt in 1..=retries {
        let mut wfc = Wfc::new(&sample, config.clone());

        if wfc.run() == RunOutcome::Complete {
            let colors = wfc.render();
            let out_sample = Sample::new(width, height, colors);
            match out_sample.save(Path::new(&output)) {
                Ok(()) => {
                    eprintln!("Saved to {} (attempt {})", output.display(), attempt);
                    return;
                }
                Err(e) => {
                    eprintln!("Error saving '{}': {}", output.display(), e);
                    process::exit(1);
                }
            }
        }

        eprintln!(
            "Attempt {}/{}: contradiction, retrying...",
            attempt, retries
        );
    }

    eprintln!(
        "Failed after {} retries - all attempts hit contradictions",
        retries
    );
    process::exit(1);
}

#[allow(clippy::type_complexity)]
fn cmd_generate_samples(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("Error creating directory '{}': {}", dir.display(), e);
        process::exit(1);
    });

    let samples: &[(&str, fn() -> Sample)] = &[
        ("pipes.png", make_pipes),
        ("maze.png", make_maze),
        ("circuits.png", make_circuits),
        ("flowers.png", make_flowers),
        ("knots.png", make_knots),
        ("stripes.png", make_stripes),
    ];

    for (filename, make) in samples {
        let path = dir.join(filename);
        make().save(&path).unwrap_or_else(|e| {
            eprintln!("Error saving '{}': {}", path.display(), e);
            process::exit(1);
        });
    }

    println!("Generated {} samples in {}/", samples.len(), dir.display());
}

fn make_pipes() -> Sample {
    let bg: Color = [32, 32, 48];
    let pipe: Color = [64, 128, 192];
    let joint: Color = [96, 192, 255];
    #[rustfmt::skip]
    let pixels = vec![
        bg,    bg,    bg,    bg,    bg,    bg,    bg,    bg,
        bg,    joint, pipe,  pipe,  pipe,  joint, bg,    bg,
        bg,    pipe,  bg,    bg,    bg,    pipe,  bg,    bg,
        bg,    pipe,  bg,    joint, pipe,  joint, pipe,  bg,
        bg,    pipe,  bg,    pipe,  bg,    bg,    pipe,  bg,
        bg,    joint, pipe,  joint, bg,    joint, pipe,  bg,
        bg,    bg,    bg,    pipe,  bg,    pipe,  bg,    bg,
        bg,    bg,    bg,    joint, pipe,  joint, bg,    bg,
    ];
    Sample::new(8, 8, pixels)
}

fn make_maze() -> Sample {
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
    Sample::new(8, 8, pixels)
}

fn make_circuits() -> Sample {
    let bg: Color = [20, 30, 20];
    let trace: Color = [50, 200, 50];
    let node: Color = [200, 200, 50];
    #[rustfmt::skip]
    let pixels = vec![
        bg,    bg,    trace, bg,    bg,    bg,    trace, bg,
        bg,    node,  trace, trace, trace, node,  trace, bg,
        trace, trace, bg,    bg,    bg,    trace, bg,    bg,
        bg,    trace, bg,    node,  trace, trace, trace, trace,
        bg,    trace, bg,    trace, bg,    bg,    bg,    bg,
        trace, trace, trace, trace, bg,    node,  trace, bg,
        bg,    bg,    bg,    bg,    bg,    trace, bg,    bg,
        bg,    bg,    trace, trace, trace, trace, bg,    bg,
    ];
    Sample::new(8, 8, pixels)
}

fn make_flowers() -> Sample {
    let grass: Color = [60, 140, 60];
    let stem: Color = [40, 100, 40];
    let petal: Color = [255, 100, 150];
    let center: Color = [255, 220, 50];
    #[rustfmt::skip]
    let pixels = vec![
        grass, grass, petal,  petal,  petal,  grass, grass, grass,
        grass, petal, petal,  center, petal,  petal, grass, grass,
        grass, petal, center, center, center, petal, grass, grass,
        grass, petal, petal,  center, petal,  petal, grass, grass,
        grass, grass, petal,  stem,   petal,  grass, grass, grass,
        grass, grass, grass,  stem,   grass,  grass, grass, grass,
        grass, grass, grass,  stem,   grass,  grass, petal, grass,
        grass, grass, grass,  stem,   grass,  petal, center, petal,
    ];
    Sample::new(8, 8, pixels)
}

fn make_knots() -> Sample {
    let bg: Color = [240, 230, 210];
    let rope: Color = [139, 90, 43];
    let shadow: Color = [100, 60, 30];
    #[rustfmt::skip]
    let pixels = vec![
        bg,     bg,     rope,   rope,   bg,     bg,     bg,     bg,
        bg,     rope,   shadow, rope,   rope,   bg,     bg,     bg,
        rope,   shadow, bg,     bg,     rope,   rope,   bg,     bg,
        rope,   bg,     bg,     bg,     bg,     rope,   rope,   bg,
        rope,   rope,   bg,     bg,     bg,     bg,     rope,   rope,
        bg,     rope,   rope,   bg,     bg,     rope,   shadow, rope,
        bg,     bg,     rope,   rope,   rope,   shadow, bg,     bg,
        bg,     bg,     bg,     rope,   rope,   bg,     bg,     bg,
    ];
    Sample::new(8, 8, pixels)
}

fn make_stripes() -> Sample {
    let c1: Color = [65, 105, 225];
    let c2: Color = [255, 255, 255];
    let c3: Color = [220, 20, 60];
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
    Sample::new(8, 8, pixels)
}
