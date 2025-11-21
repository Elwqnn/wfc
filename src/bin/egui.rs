//! WFC egui visualizer

use std::path::PathBuf;

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use wfc::{default_pipe_sample, Sample, Wfc, WfcConfig};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WFC - Wave Function Collapse",
        options,
        Box::new(|_| Ok(Box::new(App::new()))),
    )
}

struct App {
    config: WfcConfig,
    wfc: Wfc,
    sample: Sample,
    sample_path: Option<PathBuf>,
    running: bool,
    steps_per_frame: usize,
    show_entropy: bool,
    show_grid: bool,
    cell_size: f32,
    auto_restart: bool,
    error_msg: Option<String>,
}

impl App {
    fn new() -> Self {
        let sample = default_pipe_sample();
        let config = WfcConfig {
            pattern_size: 3,
            output_width: 32,
            output_height: 32,
            periodic_input: true,
            periodic_output: false,
            symmetry: true,
        };
        let wfc = Wfc::new(&sample, config.clone());

        Self {
            config,
            wfc,
            sample,
            sample_path: None,
            running: false,
            steps_per_frame: 1,
            show_entropy: true,
            show_grid: false,
            cell_size: 16.0,
            auto_restart: true,
            error_msg: None,
        }
    }

    fn rebuild(&mut self) {
        self.wfc = Wfc::new(&self.sample, self.config.clone());
        self.running = false;
    }

    fn reset(&mut self) {
        self.wfc.reset();
        self.running = false;
    }

    fn load_sample(&mut self, path: PathBuf) {
        match Sample::from_image(&path) {
            Ok(sample) => {
                self.sample = sample;
                self.sample_path = Some(path);
                self.error_msg = None;
                self.rebuild();
            }
            Err(e) => {
                self.error_msg = Some(format!("Failed to load: {}", e));
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
            .set_directory("samples")
            .pick_file()
        {
            self.load_sample(path);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Side panel with controls
        egui::SidePanel::left("controls").min_width(200.0).show(ctx, |ui| {
            ui.heading("WFC Controls");
            ui.separator();

            // Status
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.wfc.contradiction {
                    ui.colored_label(Color32::RED, "Contradiction!");
                } else if self.wfc.done {
                    ui.colored_label(Color32::GREEN, "Done");
                } else if self.running {
                    ui.colored_label(Color32::YELLOW, "Running...");
                } else {
                    ui.label("Paused");
                }
            });

            ui.horizontal(|ui| {
                ui.label("Patterns:");
                ui.label(format!("{}", self.wfc.patterns.len()));
            });

            // Error message
            if let Some(err) = &self.error_msg {
                ui.colored_label(Color32::RED, err);
            }

            ui.separator();
            ui.heading("Sample");

            if ui.button("Load Image...").clicked() {
                self.open_file_dialog();
            }

            if let Some(path) = &self.sample_path {
                ui.label(format!(
                    "{}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            } else {
                ui.label("(default pipes)");
            }

            ui.horizontal(|ui| {
                ui.label(format!("{}x{}", self.sample.width, self.sample.height));
            });

            // Sample preview
            let sample_size = 80.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(sample_size, sample_size), egui::Sense::hover());
            let rect = response.rect;
            let px_w = sample_size / self.sample.width as f32;
            let px_h = sample_size / self.sample.height as f32;

            for y in 0..self.sample.height {
                for x in 0..self.sample.width {
                    let color = self.sample.get(x, y);
                    let pos = rect.min + Vec2::new(x as f32 * px_w, y as f32 * px_h);
                    painter.rect_filled(
                        Rect::from_min_size(pos, Vec2::new(px_w, px_h)),
                        0.0,
                        Color32::from_rgb(color[0], color[1], color[2]),
                    );
                }
            }

            ui.separator();
            ui.heading("Configuration");

            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Pattern size:");
                if ui
                    .add(egui::Slider::new(&mut self.config.pattern_size, 2..=4))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui
                    .add(egui::Slider::new(&mut self.config.output_width, 8..=64))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Height:");
                if ui
                    .add(egui::Slider::new(&mut self.config.output_height, 8..=64))
                    .changed()
                {
                    changed = true;
                }
            });

            if ui.checkbox(&mut self.config.symmetry, "Symmetry").changed() {
                changed = true;
            }

            if ui
                .checkbox(&mut self.config.periodic_output, "Periodic output")
                .changed()
            {
                changed = true;
            }

            if changed {
                self.rebuild();
            }

            ui.separator();
            ui.heading("Playback");

            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=100).logarithmic(true));
            });

            ui.horizontal(|ui| {
                if ui
                    .button(if self.running { "⏸ Pause" } else { "▶ Run" })
                    .clicked()
                {
                    self.running = !self.running;
                }
                if ui.button("⏭ Step").clicked() {
                    self.wfc.step();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🔄 Reset").clicked() {
                    self.reset();
                }
                if ui.button("🎲 New").clicked() {
                    self.rebuild();
                }
            });

            ui.checkbox(&mut self.auto_restart, "Auto-restart on contradiction");

            ui.separator();
            ui.heading("Visualization");

            ui.checkbox(&mut self.show_entropy, "Show entropy heatmap");
            ui.checkbox(&mut self.show_grid, "Show grid lines");

            ui.horizontal(|ui| {
                ui.label("Cell size:");
                ui.add(egui::Slider::new(&mut self.cell_size, 4.0..=32.0));
            });
        });

        // Main canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            // Run steps if playing
            if self.running && !self.wfc.done {
                if self.wfc.contradiction {
                    if self.auto_restart {
                        self.wfc.reset();
                    } else {
                        self.running = false;
                    }
                } else {
                    for _ in 0..self.steps_per_frame {
                        if !self.wfc.step() {
                            break;
                        }
                    }
                }
                ctx.request_repaint();
            }

            let canvas_width = self.config.output_width as f32 * self.cell_size;
            let canvas_height = self.config.output_height as f32 * self.cell_size;

            egui::ScrollArea::both().show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    Vec2::new(canvas_width, canvas_height),
                    egui::Sense::hover(),
                );
                let rect = response.rect;

                // Draw cells
                for y in 0..self.config.output_height {
                    for x in 0..self.config.output_width {
                        let pos = rect.min
                            + Vec2::new(x as f32 * self.cell_size, y as f32 * self.cell_size);
                        let cell_rect = Rect::from_min_size(pos, Vec2::splat(self.cell_size));

                        // Base color
                        let color = self.wfc.get_color(x, y);
                        let mut base = Color32::from_rgb(color[0], color[1], color[2]);

                        // Entropy overlay
                        if self.show_entropy && !self.wfc.is_collapsed(x, y) {
                            let entropy = self.wfc.normalized_entropy(x, y);
                            // Blue (low entropy) to Red (high entropy)
                            let r = (entropy * 255.0) as u8;
                            let b = ((1.0 - entropy) * 255.0) as u8;
                            let overlay = Color32::from_rgba_unmultiplied(r, 64, b, 128);
                            base = blend_colors(base, overlay);
                        }

                        painter.rect_filled(cell_rect, 0.0, base);

                        // Highlight last collapsed
                        if let Some((lx, ly)) = self.wfc.last_collapsed
                            && x == lx
                            && y == ly
                        {
                            let inset = cell_rect.shrink(1.0);
                            painter.rect_stroke(
                                inset,
                                0.0,
                                Stroke::new(2.0, Color32::YELLOW),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                }

                // Grid lines
                if self.show_grid {
                    let stroke = Stroke::new(1.0, Color32::from_gray(64));
                    for x in 0..=self.config.output_width {
                        let px = rect.min.x + x as f32 * self.cell_size;
                        painter.line_segment(
                            [Pos2::new(px, rect.min.y), Pos2::new(px, rect.max.y)],
                            stroke,
                        );
                    }
                    for y in 0..=self.config.output_height {
                        let py = rect.min.y + y as f32 * self.cell_size;
                        painter.line_segment(
                            [Pos2::new(rect.min.x, py), Pos2::new(rect.max.x, py)],
                            stroke,
                        );
                    }
                }
            });
        });

        // Keep repainting - continuous when running, periodic when idle (helps Wayland responsiveness)
        if self.running {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }
}

/// Blend two colors (simple alpha blend)
fn blend_colors(base: Color32, overlay: Color32) -> Color32 {
    let alpha = overlay.a() as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Color32::from_rgb(
        (base.r() as f32 * inv_alpha + overlay.r() as f32 * alpha) as u8,
        (base.g() as f32 * inv_alpha + overlay.g() as f32 * alpha) as u8,
        (base.b() as f32 * inv_alpha + overlay.b() as f32 * alpha) as u8,
    )
}
