use crate::gui::{GuiState, Tool};
use egui::Ui;

#[derive(Debug, Clone)]
pub enum ToolPanelAction {
    ZoomIn,
    ZoomOut,
    FitToWindow,
    ExportSvg,
    ExportPng,
    ExportPdf,
    None,
}

/// Tool panel for map interaction tools
pub struct ToolPanel;

impl ToolPanel {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut Ui, gui_state: &mut GuiState) -> ToolPanelAction {
        let mut action = ToolPanelAction::None;
        ui.heading("Tools");
        ui.separator();
        
        // Tool selection
        ui.group(|ui| {
            ui.label("Navigation:");
            
            if ui.selectable_label(
                matches!(gui_state.current_tool, Tool::Pan),
                "🤚 Pan"
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
        
        ui.separator();
        
        // Selection tools
        ui.group(|ui| {
            ui.label("Selection:");
            
            if ui.selectable_label(
                matches!(gui_state.current_tool, Tool::Select),
                "👆 Select"
            ).clicked() {
                gui_state.current_tool = Tool::Select;
            }
        });
        
        ui.separator();
        
        // Zoom controls
        ui.group(|ui| {
            ui.label("Zoom:");
            
            ui.horizontal(|ui| {
                if ui.button("🔍+").clicked() {
                    action = ToolPanelAction::ZoomIn;
                }
                if ui.button("🔍-").clicked() {
                    action = ToolPanelAction::ZoomOut;
                }
            });
            
            ui.label(format!("Level: {:.1}x", gui_state.zoom_level));
            
            if ui.button("🎯 Fit to Window").clicked() {
                action = ToolPanelAction::FitToWindow;
            }
        });
        
        ui.separator();
        
        // Layer controls
        ui.group(|ui| {
            ui.label("Layers:");
            
            ui.checkbox(&mut true, "🗺 Base Map");
            ui.checkbox(&mut true, "🏠 Buildings");
            ui.checkbox(&mut true, "🛣 Roads");
            ui.checkbox(&mut true, "💧 Water");
            ui.checkbox(&mut true, "🌳 Landuse");
            ui.checkbox(&mut false, "📍 POIs");
            ui.checkbox(&mut false, "🚶 GPX Tracks");
        });
        
        ui.separator();
        
        // Export section
        ui.group(|ui| {
            ui.label("Export:");
            
            if ui.button("📄 Export SVG").clicked() {
                action = ToolPanelAction::ExportSvg;
            }
            
            if ui.button("🖼 Export PNG").clicked() {
                action = ToolPanelAction::ExportPng;
            }
            
            if ui.button("📑 Export PDF").clicked() {
                action = ToolPanelAction::ExportPdf;
            }
        });
        
        action
    }
}

impl Default for ToolPanel {
    fn default() -> Self {
        Self::new()
    }
}
