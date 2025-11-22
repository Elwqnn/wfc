use eframe::egui;
use gif::{Encoder, Frame, Repeat};

use crate::Sample;

use super::App;

impl App {
    pub fn save_output(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name("output.png")
            .save_file()
        else {
            return;
        };

        let colors = self.wfc.render();
        let w = self.config.output_width;
        let h = self.config.output_height;

        let result = if self.export_scale == 1 {
            Sample::new(w, h, colors).save(&path)
        } else {
            let mut img = image::RgbImage::new(w as u32, h as u32);
            for y in 0..h {
                for x in 0..w {
                    let c = colors[y * w + x];
                    img.put_pixel(x as u32, y as u32, image::Rgb(c));
                }
            }
            let scaled = image::imageops::resize(
                &img,
                w as u32 * self.export_scale,
                h as u32 * self.export_scale,
                image::imageops::FilterType::Nearest,
            );
            scaled.save(&path).map_err(|e| e.to_string())
        };

        match result {
            Ok(_) => self.success_msg = Some("Image saved successfully".to_string()),
            Err(e) => self.error_msg = Some(format!("Failed to save: {}", e)),
        }
    }

    pub fn start_save_gif(&mut self) {
        if self.gif_frames.is_empty() {
            self.error_msg = Some("No frames to save".to_string());
            return;
        }

        let Some(path) = rfd::FileDialog::new()
            .add_filter("GIF", &["gif"])
            .set_file_name("wfc-animation.gif")
            .save_file()
        else {
            return;
        };

        let w = (self.config.output_width as u32 * self.export_scale) as u16;
        let h = (self.config.output_height as u32 * self.export_scale) as u16;

        match std::fs::File::create(&path)
            .map_err(|e| e.to_string())
            .and_then(|f| Encoder::new(f, w, h, &[]).map_err(|e| e.to_string()))
            .and_then(|mut e| {
                e.set_repeat(Repeat::Infinite)
                    .map(|_| e)
                    .map_err(|e| e.to_string())
            }) {
            Ok(encoder) => {
                self.gif_encoder = Some(encoder);
                self.gif_save_path = Some(path);
                self.saving_gif = true;
                self.gif_save_progress = 0;
                self.gif_save_cancel = false;
            }
            Err(e) => self.error_msg = Some(format!("Failed to initialize GIF: {}", e)),
        }
    }

    pub fn process_gif_saving(&mut self, ctx: &egui::Context) {
        if self.gif_save_cancel {
            self.error_msg = Some("GIF save cancelled".to_string());
            self.saving_gif = false;
            self.gif_encoder = None;
            self.gif_save_path = None;
            return;
        }

        let Some(encoder) = &mut self.gif_encoder else {
            return;
        };

        let idx = self.gif_save_progress;
        if idx >= self.gif_frames.len() {
            self.saving_gif = false;
            self.gif_encoder = None;
            if let Some(path) = &self.gif_save_path {
                self.success_msg = Some(format!("GIF saved to {}", path.display()));
            }
            self.gif_save_path = None;
            return;
        }

        let w = self.config.output_width as u32;
        let h = self.config.output_height as u32;
        let scaled_w = (w * self.export_scale) as u16;
        let scaled_h = (h * self.export_scale) as u16;

        let scaled_frame = if self.export_scale == 1 {
            self.gif_frames[idx].clone()
        } else {
            let img = image::RgbaImage::from_raw(w, h, self.gif_frames[idx].clone()).unwrap();
            image::imageops::resize(
                &img,
                scaled_w as u32,
                scaled_h as u32,
                image::imageops::FilterType::Nearest,
            )
            .into_raw()
        };

        let mut scaled_frame_mut = scaled_frame;
        let mut frame = Frame::from_rgba_speed(scaled_w, scaled_h, &mut scaled_frame_mut, 10);
        frame.delay = self.gif_frame_delay;

        if let Err(e) = encoder.write_frame(&frame) {
            self.error_msg = Some(format!("Failed to write frame: {}", e));
            self.saving_gif = false;
            self.gif_encoder = None;
            self.gif_save_path = None;
        } else {
            self.gif_save_progress = idx + 1;
            ctx.request_repaint();
        }
    }

    pub fn show_gif_saving_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("Saving GIF")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!(
                        "Processing frame {} of {}...",
                        self.gif_save_progress,
                        self.gif_frames.len()
                    ));

                    let progress = self.gif_save_progress as f32 / self.gif_frames.len() as f32;
                    ui.add(egui::ProgressBar::new(progress).show_percentage());

                    ui.add_space(10.0);
                    if ui.button("Cancel").clicked() {
                        self.gif_save_cancel = true;
                    }
                    ui.add_space(10.0);
                });
            });
        ctx.request_repaint();
    }

    pub fn scale_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Scale:");
            ui.add(egui::Slider::new(&mut self.export_scale, 1..=32).logarithmic(true));
            let w = self.config.output_width as u32 * self.export_scale;
            let h = self.config.output_height as u32 * self.export_scale;
            ui.label(format!("{}x{}", w, h));
        });
    }
}
