use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use wfc_core::{Boundary, StepOutcome};

use super::App;

fn config_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut usize,
    range: std::ops::RangeInclusive<usize>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.add(egui::Slider::new(value, range)).changed() {
            changed = true;
        }
    });
    changed
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.export.saving_gif {
            self.process_gif_saving(ctx);
            self.show_gif_saving_modal(ctx);
            return;
        }

        egui::SidePanel::left("controls")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("WFC Controls");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    if self.wfc.has_contradiction() {
                        ui.colored_label(Color32::RED, "Contradiction!");
                    } else if self.wfc.is_done() {
                        ui.colored_label(Color32::GREEN, "Done");
                    } else if self.playback.running {
                        ui.colored_label(Color32::YELLOW, "Running...");
                    } else {
                        ui.label("Paused");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Patterns:");
                    ui.label(self.wfc.num_patterns().to_string());
                });

                if let Some(err) = &self.messages.error {
                    ui.colored_label(Color32::RED, err);
                }
                if let Some(msg) = &self.messages.success {
                    ui.colored_label(Color32::GREEN, msg);
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

                ui.label(format!("{}x{}", self.sample.width, self.sample.height));
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

                let mut config = self.wfc.config().clone();
                let mut changed = false;

                changed |= config_slider(ui, "Pattern size:", &mut config.pattern_size, 2..=4);
                changed |= config_slider(ui, "Width:", &mut config.output_width, 8..=128);
                changed |= config_slider(ui, "Height:", &mut config.output_height, 8..=128);
                changed |= ui.checkbox(&mut config.symmetry, "Symmetry").changed();

                let boundary_label = match config.boundary {
                    Boundary::Fixed => "Fixed",
                    Boundary::PeriodicX => "Periodic X",
                    Boundary::PeriodicY => "Periodic Y",
                    Boundary::Periodic => "Periodic",
                };
                egui::ComboBox::from_label("Boundary")
                    .selected_text(boundary_label)
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(&mut config.boundary, Boundary::Fixed, "Fixed")
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut config.boundary,
                                Boundary::PeriodicX,
                                "Periodic X",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut config.boundary,
                                Boundary::PeriodicY,
                                "Periodic Y",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(&mut config.boundary, Boundary::Periodic, "Periodic")
                            .changed();
                    });

                changed |= ui
                    .checkbox(&mut config.ground, "Ground (preserve verticality)")
                    .changed();
                changed |= ui
                    .checkbox(&mut config.sides, "Sides (preserve horizontality)")
                    .changed();

                if changed {
                    self.rebuild_with_config(config);
                }

                ui.separator();
                ui.heading("Playback");

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(
                        egui::Slider::new(&mut self.playback.steps_per_frame, 1..=100)
                            .logarithmic(true),
                    );
                });

                ui.horizontal(|ui| {
                    let is_finished = self.wfc.is_done() || self.wfc.has_contradiction();
                    let button_label = if is_finished {
                        "🔄 Rerun"
                    } else if self.playback.running {
                        "⏸ Pause"
                    } else {
                        "▶ Run"
                    };
                    if ui.button(button_label).clicked() {
                        if is_finished {
                            self.reset();
                            self.playback.running = true;
                        } else {
                            self.playback.running = !self.playback.running;
                        }
                    }
                    if ui.button("⏭ Step").clicked() {
                        let _ = self.wfc.step();
                        self.capture_frame();
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("🎲 New").clicked() {
                        self.rebuild();
                    }
                });

                ui.checkbox(
                    &mut self.playback.auto_restart,
                    "Auto-restart on contradiction",
                );

                ui.separator();
                ui.heading("Export");

                ui.label(egui::RichText::new("Image (PNG)").strong());
                self.scale_ui(ui);
                if ui.button("💾 Save PNG").clicked() {
                    self.save_output();
                }

                ui.add_space(8.0);

                ui.label(egui::RichText::new("Animation (GIF)").strong());
                self.scale_ui(ui);
                ui.horizontal(|ui| {
                    ui.label("Delay:");
                    ui.add(
                        egui::Slider::new(&mut self.export.gif_frame_delay, 1..=100).suffix(" cs"),
                    );
                    ui.label(format!(
                        "({:.1} fps)",
                        100.0 / self.export.gif_frame_delay as f32
                    ));
                });
                if !self.export.gif_frames.is_empty() {
                    ui.label(format!("{} frames recorded", self.export.gif_frames.len()));
                }
                if ui.button("🎞 Save GIF").clicked() {
                    self.start_save_gif();
                }

                ui.separator();
                ui.heading("Visualization");

                ui.checkbox(&mut self.show_grid, "Show grid lines");

                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    if ui.button("Fit").clicked() {
                        self.camera.zoom = 0.0;
                        self.camera.pan_offset = Vec2::ZERO;
                    }
                    if ui.button("100%").clicked() {
                        self.camera.zoom = 1.0;
                        self.camera.pan_offset = Vec2::ZERO;
                    }
                    ui.label(if self.camera.zoom > 0.0 {
                        format!("{:.0}%", self.camera.zoom * 100.0)
                    } else {
                        "Auto".to_string()
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.playback.running && !self.wfc.is_done() {
                if self.wfc.has_contradiction() {
                    if self.playback.auto_restart {
                        self.wfc.reset();
                        self.capture_frame();
                    } else {
                        self.playback.running = false;
                    }
                } else {
                    for _ in 0..self.playback.steps_per_frame {
                        if self.wfc.step() != StepOutcome::Progressed {
                            break;
                        }
                        self.capture_frame();
                    }
                }
                ctx.request_repaint();
            }

            let output_width = self.wfc.config().output_width;
            let output_height = self.wfc.config().output_height;
            let available_size = ui.available_size();

            let (response, painter) =
                ui.allocate_painter(available_size, egui::Sense::click_and_drag());

            // Calculate current actual_zoom for input handling
            let current_actual_zoom = if self.camera.zoom <= 0.0 {
                let zoom_w = available_size.x / output_width as f32;
                let zoom_h = available_size.y / output_height as f32;
                zoom_w.min(zoom_h) * 0.95
            } else {
                self.camera.zoom * self.camera.cell_size
            };

            // Handle mouse wheel zoom centered on cursor BEFORE rendering calculations
            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    let zoom_factor = 1.0 + scroll * 0.002;

                    // Transition from auto-fit to manual zoom
                    if self.camera.zoom <= 0.0 {
                        self.camera.zoom = current_actual_zoom / self.camera.cell_size;
                    }

                    // Get cursor position
                    if let Some(cursor_pos) = response.hover_pos() {
                        let canvas_width = output_width as f32 * current_actual_zoom;
                        let canvas_height = output_height as f32 * current_actual_zoom;
                        let offset_x = (available_size.x - canvas_width) * 0.5;
                        let offset_y = (available_size.y - canvas_height) * 0.5;

                        let current_origin = response.rect.min
                            + Vec2::new(offset_x.max(0.0), offset_y.max(0.0))
                            + self.camera.pan_offset;
                        let cursor_rel = cursor_pos - current_origin;

                        let new_zoom = (self.camera.zoom * zoom_factor).clamp(0.1, 10.0);
                        let zoom_ratio = new_zoom / self.camera.zoom;
                        self.camera.pan_offset += cursor_rel * (1.0 - zoom_ratio);
                        self.camera.zoom = new_zoom;
                    } else {
                        self.camera.zoom = (self.camera.zoom * zoom_factor).clamp(0.1, 10.0);
                    }
                }
            }

            // Handle middle mouse button drag for panning
            if response.dragged_by(egui::PointerButton::Middle) {
                self.camera.pan_offset += response.drag_delta();
            }

            // Now calculate final actual_zoom for rendering with updated zoom value
            let actual_zoom = if self.camera.zoom <= 0.0 {
                let zoom_w = available_size.x / output_width as f32;
                let zoom_h = available_size.y / output_height as f32;
                zoom_w.min(zoom_h) * 0.95
            } else {
                self.camera.zoom * self.camera.cell_size
            };

            let canvas_width = output_width as f32 * actual_zoom;
            let canvas_height = output_height as f32 * actual_zoom;

            // Center the canvas
            let offset_x = (available_size.x - canvas_width) * 0.5;
            let offset_y = (available_size.y - canvas_height) * 0.5;

            let canvas_origin = response.rect.min
                + Vec2::new(offset_x.max(0.0), offset_y.max(0.0))
                + self.camera.pan_offset;

            for y in 0..output_height {
                for x in 0..output_width {
                    let pos =
                        canvas_origin + Vec2::new(x as f32 * actual_zoom, y as f32 * actual_zoom);
                    let cell_rect = Rect::from_min_size(pos, Vec2::splat(actual_zoom));

                    let color = self.wfc.get_color(x, y);
                    let base = Color32::from_rgb(color[0], color[1], color[2]);

                    painter.rect_filled(cell_rect, 0.0, base);

                    if let Some((lx, ly)) = self.wfc.last_collapsed()
                        && x == lx
                        && y == ly
                    {
                        painter.rect_stroke(
                            cell_rect.shrink(1.0),
                            0.0,
                            Stroke::new(4.0, Color32::RED),
                            egui::StrokeKind::Middle,
                        );
                    }
                }
            }

            if self.show_grid {
                let stroke = Stroke::new(1.0, Color32::from_gray(64));
                for x in 0..=output_width {
                    let px = canvas_origin.x + x as f32 * actual_zoom;
                    painter.line_segment(
                        [
                            Pos2::new(px, canvas_origin.y),
                            Pos2::new(px, canvas_origin.y + canvas_height),
                        ],
                        stroke,
                    );
                }
                for y in 0..=output_height {
                    let py = canvas_origin.y + y as f32 * actual_zoom;
                    painter.line_segment(
                        [
                            Pos2::new(canvas_origin.x, py),
                            Pos2::new(canvas_origin.x + canvas_width, py),
                        ],
                        stroke,
                    );
                }
            }
        });

        if self.playback.running {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }
}
