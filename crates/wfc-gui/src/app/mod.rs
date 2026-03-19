use std::path::PathBuf;

use eframe::egui::Vec2;
use gif::Encoder;

use wfc_core::{Config, Sample, Wfc, default_pipe_sample};

pub mod export;
pub mod ui;

pub struct CameraState {
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub cell_size: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            cell_size: 16.0,
        }
    }
}

pub struct ExportState {
    pub gif_frames: Vec<Vec<u8>>,
    pub gif_frame_delay: u16,
    pub export_scale: u32,
    pub saving_gif: bool,
    pub gif_save_progress: usize,
    pub gif_save_cancel: bool,
    pub gif_save_path: Option<PathBuf>,
    pub gif_encoder: Option<Encoder<std::fs::File>>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            gif_frames: Vec::new(),
            gif_frame_delay: 5,
            export_scale: 1,
            saving_gif: false,
            gif_save_progress: 0,
            gif_save_cancel: false,
            gif_save_path: None,
            gif_encoder: None,
        }
    }
}

pub struct PlaybackState {
    pub running: bool,
    pub steps_per_frame: usize,
    pub auto_restart: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            running: false,
            steps_per_frame: 1,
            auto_restart: true,
        }
    }
}

#[derive(Default)]
pub struct Messages {
    pub error: Option<String>,
    pub success: Option<String>,
}

pub struct App {
    pub wfc: Wfc,
    pub sample: Sample,
    pub sample_path: Option<PathBuf>,
    pub show_grid: bool,
    pub camera: CameraState,
    pub export: ExportState,
    pub playback: PlaybackState,
    pub messages: Messages,
}

impl Default for App {
    fn default() -> Self {
        let sample = default_pipe_sample();
        let config = Config::default();
        let wfc = Wfc::new(&sample, config);

        let mut app = Self {
            wfc,
            sample,
            sample_path: None,
            show_grid: false,
            camera: CameraState::default(),
            export: ExportState::default(),
            playback: PlaybackState::default(),
            messages: Messages::default(),
        };
        app.capture_frame();
        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(&self) -> &Config {
        self.wfc.config()
    }

    pub fn rebuild_with_config(&mut self, config: Config) {
        self.wfc = Wfc::new(&self.sample, config);
        self.playback.running = false;
        self.export.gif_frames.clear();
        self.capture_frame();
    }

    pub fn rebuild(&mut self) {
        self.rebuild_with_config(self.wfc.config().clone());
    }

    pub fn reset(&mut self) {
        self.wfc.reset();
        self.playback.running = false;
        self.export.gif_frames.clear();
        self.capture_frame();
    }

    pub fn load_sample(&mut self, path: PathBuf) {
        match Sample::from_image(&path) {
            Ok(sample) => {
                self.sample = sample;
                self.sample_path = Some(path);
                self.messages.error = None;
                self.messages.success = Some("Sample loaded successfully".to_string());
                self.export.gif_frames.clear();
                self.rebuild();
            }
            Err(e) => self.messages.error = Some(format!("Failed to load: {}", e)),
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
        let config = self.wfc.config();
        let w = config.output_width;
        let h = config.output_height;

        let mut frame_data = Vec::with_capacity(w * h * 4);
        for color in colors {
            frame_data.push(color[0]);
            frame_data.push(color[1]);
            frame_data.push(color[2]);
            frame_data.push(255);
        }

        self.export.gif_frames.push(frame_data);
    }
}
