use crate::gui::{GuiState, Tool};
use egui::Ui;

#[derive(Debug, Clone)]
pub enum ToolbarAction {
    ZoomIn,
    ZoomOut,
    FitToWindow,
    ExportSvg,
    ExportPng,
    ExportPdf,
    None,
}

/// Horizontal toolbar for map interaction tools
pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut Ui, gui_state: &mut GuiState) -> ToolbarAction {
        let mut action = ToolbarAction::None;
        
        ui.horizontal(|ui| {
            // Navigation tools
            ui.group(|ui| {
                ui.label("Navigation:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(
                        matches!(gui_state.current_tool, Tool::Pan),
                        "🔀 Pan"
                    ).clicked() {
                        gui_state.current_tool = Tool::Pan;
                    }
                    
                    if ui.selectable_label(
                        matches!(gui_state.current_tool, Tool::RectangleZoom),
                        "🔲 Rectangle Zoom"
                    ).clicked() {
                        gui_state.current_tool = Tool::RectangleZoom;
                    }
                });
            });
            
            ui.separator();
            
            // Measurement tools
            ui.group(|ui| {
                ui.label("Measurement:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(
                        matches!(gui_state.current_tool, Tool::Measure),
                        "📏 Measure"
                    ).clicked() {
                        gui_state.current_tool = Tool::Measure;
                    }
                    
                    if ui.selectable_label(
                        matches!(gui_state.current_tool, Tool::Select),
                        "👆 Select"
                    ).clicked() {
                        gui_state.current_tool = Tool::Select;
                    }
                });
            });
            
            ui.separator();
            
            // Zoom controls
            ui.group(|ui| {
                ui.label("Zoom:");
                ui.horizontal(|ui| {
                    if ui.button("🔍+").clicked() {
                        action = ToolbarAction::ZoomIn;
                    }
                    if ui.button("🔍-").clicked() {
                        action = ToolbarAction::ZoomOut;
                    }
                    if ui.button("🎯 Fit").clicked() {
                        action = ToolbarAction::FitToWindow;
                    }
                    ui.label(format!("{:.1}x", gui_state.zoom_level / 10000.0));
                });
            });
            
            ui.separator();
            
            // Layer controls
            ui.group(|ui| {
                ui.label("Layers:");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut true, "🗺 Map");
                    ui.checkbox(&mut gui_state.show_buildings, "🏠 Buildings");
                    ui.checkbox(&mut gui_state.show_roads, "🛣 Roads");
                    ui.checkbox(&mut gui_state.show_water, "💧 Water");
                    ui.checkbox(&mut gui_state.show_landuse, "🌳 Land");
                    ui.checkbox(&mut gui_state.show_pois, "📍 POIs");
                    ui.checkbox(&mut gui_state.show_gpx, "🚶 GPX");
                });
            });
            
            ui.separator();
            
            // Export section
            ui.group(|ui| {
                ui.label("Export:");
                ui.horizontal(|ui| {
                    if ui.button("📄 SVG").clicked() {
                        action = ToolbarAction::ExportSvg;
                    }
                    
                    if ui.button("🖼 PNG").clicked() {
                        action = ToolbarAction::ExportPng;
                    }
                    
                    if ui.button("📑 PDF").clicked() {
                        action = ToolbarAction::ExportPdf;
                    }
                });
            });
        });
        
        action
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
