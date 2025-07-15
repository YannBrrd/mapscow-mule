/// Custom widgets for the map application
use egui::{Ui, Response, Vec2, Pos2, Rect, Color32, Stroke};

/// A custom color picker widget with better UX
pub struct ColorPicker {
    pub color: Color32,
    pub open: bool,
}

impl ColorPicker {
    pub fn new(color: Color32) -> Self {
        Self { color, open: false }
    }
    
    pub fn show(&mut self, ui: &mut Ui, text: &str) -> Response {
        let response = ui.horizontal(|ui| {
            // Color preview button
            let color_button_size = Vec2::splat(20.0);
            let (rect, response) = ui.allocate_exact_size(color_button_size, egui::Sense::click());
            
            if response.clicked() {
                self.open = !self.open;
            }
            
            // Draw color preview
            ui.painter().rect_filled(rect, 2.0, self.color);
            ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::BLACK));
            
            ui.label(text);
            
            response
        }).inner;
        
        // Show color picker popup
        if self.open {
            egui::Window::new("Color Picker")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    let mut rgb = [
                        self.color.r() as f32 / 255.0,
                        self.color.g() as f32 / 255.0,
                        self.color.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut rgb).changed() {
                        self.color = Color32::from_rgb(
                            (rgb[0] * 255.0) as u8,
                            (rgb[1] * 255.0) as u8,
                            (rgb[2] * 255.0) as u8,
                        );
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.open = false;
                        }
                    });
                });
        }
        
        response
    }
}

/// A minimap widget showing the current viewport
pub struct MiniMap {
    size: Vec2,
}

impl MiniMap {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        
        // Draw minimap background
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(240, 240, 240));
        ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::BLACK));
        
        // TODO: Draw simplified map content
        // For now, just draw a placeholder
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "MiniMap",
            egui::FontId::proportional(10.0),
            Color32::GRAY,
        );
        
        response
    }
}

/// A coordinate display widget
pub struct CoordinateDisplay {
    pub lat: f64,
    pub lon: f64,
}

impl CoordinateDisplay {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("📍");
            ui.label(format!("Lat: {:.6}°", self.lat));
            ui.label(format!("Lon: {:.6}°", self.lon));
        });
    }
}

/// A scale bar widget for the map
pub struct ScaleBar {
    pub scale: f64, // pixels per meter
    pub width: f32,
}

impl ScaleBar {
    pub fn new(scale: f64, width: f32) -> Self {
        Self { scale, width }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let height = 20.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(self.width, height), egui::Sense::hover());
        
        // Calculate distance represented by the scale bar
        let distance_meters = self.width as f64 / self.scale;
        let (distance, unit) = if distance_meters < 1000.0 {
            (distance_meters, "m")
        } else {
            (distance_meters / 1000.0, "km")
        };
        
        // Draw scale bar
        let bar_rect = Rect::from_min_size(
            rect.min + Vec2::new(0.0, height - 8.0),
            Vec2::new(self.width, 4.0)
        );
        
        ui.painter().rect_filled(bar_rect, 0.0, Color32::BLACK);
        
        // Draw tick marks
        for i in 0..=4 {
            let x = rect.min.x + (i as f32 * self.width / 4.0);
            let y1 = rect.min.y + height - 10.0;
            let y2 = rect.min.y + height - 2.0;
            
            ui.painter().line_segment(
                [Pos2::new(x, y1), Pos2::new(x, y2)],
                Stroke::new(1.0, Color32::BLACK)
            );
        }
        
        // Draw distance text
        ui.painter().text(
            rect.center() - Vec2::new(0.0, 8.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.0} {}", distance, unit),
            egui::FontId::proportional(10.0),
            Color32::BLACK,
        );
        
        response
    }
}

/// A progress widget for long-running operations
pub struct ProgressWidget {
    pub progress: f32, // 0.0 to 1.0
    pub text: String,
    pub show_percentage: bool,
}

impl ProgressWidget {
    pub fn new(progress: f32, text: String) -> Self {
        Self {
            progress,
            text,
            show_percentage: true,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label(&self.text);
            
            let progress_bar = egui::ProgressBar::new(self.progress);
            let progress_bar = if self.show_percentage {
                progress_bar.show_percentage()
            } else {
                progress_bar
            };
            
            ui.add(progress_bar)
        }).inner
    }
}
