use std::path::PathBuf;

use eframe::egui::Vec2;
use gif::Encoder;

use crate::{Sample, Wfc, WfcConfig, default_pipe_sample};

pub mod export;
pub mod ui;

pub struct App {
    pub config: WfcConfig,
    pub wfc: Wfc,
    pub sample: Sample,
    pub sample_path: Option<PathBuf>,
    pub running: bool,
    pub steps_per_frame: usize,
    pub show_grid: bool,
    pub cell_size: f32,
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub auto_restart: bool,
    pub error_msg: Option<String>,
    pub success_msg: Option<String>,
    pub gif_frames: Vec<Vec<u8>>,
    pub gif_frame_delay: u16,
    pub export_scale: u32,
    pub saving_gif: bool,
    pub gif_save_progress: usize,
    pub gif_save_cancel: bool,
    pub gif_save_path: Option<PathBuf>,
    pub gif_encoder: Option<Encoder<std::fs::File>>,
}

impl Default for App {
    fn default() -> Self {
        let sample = default_pipe_sample();
        let config = WfcConfig {
            pattern_size: 3,
            output_width: 32,
            output_height: 32,
            periodic_input: true,
            periodic_output: false,
            symmetry: true,
            ground: false,
            sides: false,
        };
        let wfc = Wfc::new(&sample, config.clone());

        let mut app = Self {
            config,
            wfc,
            sample,
            sample_path: None,
            running: false,
            steps_per_frame: 1,
            show_grid: false,
            cell_size: 16.0,
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            auto_restart: true,
            error_msg: None,
            success_msg: None,
            gif_frames: Vec::new(),
            gif_frame_delay: 5,
            export_scale: 1,
            saving_gif: false,
            gif_save_progress: 0,
            gif_save_cancel: false,
            gif_save_path: None,
            gif_encoder: None,
        };
        app.capture_frame();
        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild(&mut self) {
        self.wfc = Wfc::new(&self.sample, self.config.clone());
        self.running = false;
        self.gif_frames.clear();
        self.capture_frame();
    }

    pub fn reset(&mut self) {
        self.wfc.reset();
        self.running = false;
        self.gif_frames.clear();
        self.capture_frame();
    }

    pub fn load_sample(&mut self, path: PathBuf) {
        match Sample::from_image(&path) {
            Ok(sample) => {
                self.sample = sample;
                self.sample_path = Some(path);
                self.error_msg = None;
                self.success_msg = Some("Sample loaded successfully".to_string());
                self.gif_frames.clear();
                self.rebuild();
            }
            Err(e) => self.error_msg = Some(format!("Failed to load: {}", e)),
        }
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
            .set_directory("samples")
            .pick_file()
        {
            self.load_sample(path);
        }
    }

    pub fn capture_frame(&mut self) {
        let colors = self.wfc.render();
        let w = self.config.output_width;
        let h = self.config.output_height;

        let mut frame_data = Vec::with_capacity(w * h * 4);
        for color in colors {
            frame_data.push(color[0]);
            frame_data.push(color[1]);
            frame_data.push(color[2]);
            frame_data.push(255);
        }

        self.gif_frames.push(frame_data);
    }
}
