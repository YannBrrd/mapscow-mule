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
            
            // Selection tools
            ui.group(|ui| {
                ui.label("Selection:");
                ui.horizontal(|ui| {
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
