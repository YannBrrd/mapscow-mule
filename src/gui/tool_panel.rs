use crate::gui::{GuiState, Tool};
use egui::Ui;

/// Tool panel for map interaction tools
pub struct ToolPanel;

impl ToolPanel {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut Ui, gui_state: &mut GuiState) {
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
                matches!(gui_state.current_tool, Tool::Zoom),
                "🔍 Zoom"
            ).clicked() {
                gui_state.current_tool = Tool::Zoom;
            }
            
            if ui.selectable_label(
                matches!(gui_state.current_tool, Tool::RectangleZoom),
                "🔲 Rectangle Zoom"
            ).clicked() {
                gui_state.current_tool = Tool::RectangleZoom;
            }
        });
        
        ui.separator();
        
        // Measurement tools
        ui.group(|ui| {
            ui.label("Measurement:");
            
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
        
        ui.separator();
        
        // Zoom controls
        ui.group(|ui| {
            ui.label("Zoom:");
            
            ui.horizontal(|ui| {
                if ui.button("🔍+").clicked() {
                    gui_state.zoom_level *= 1.2;
                }
                if ui.button("🔍-").clicked() {
                    gui_state.zoom_level /= 1.2;
                }
            });
            
            ui.label(format!("Level: {:.1}x", gui_state.zoom_level));
            
            if ui.button("🎯 Fit to Window").clicked() {
                // TODO: Implement zoom to fit
                gui_state.zoom_level = 1.0;
                gui_state.pan_offset = (0.0, 0.0);
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
                // TODO: Trigger SVG export
            }
            
            if ui.button("🖼 Export PNG").clicked() {
                // TODO: Trigger PNG export
            }
            
            if ui.button("📑 Export PDF").clicked() {
                // TODO: Trigger PDF export
            }
        });
        
        ui.separator();
        
        // Map information
        ui.group(|ui| {
            ui.label("Map Info:");
            ui.label("📍 Features: 0");
            ui.label("📊 Nodes: 0");
            ui.label("🛣 Ways: 0");
            ui.label("🔗 Relations: 0");
        });
    }
}

impl Default for ToolPanel {
    fn default() -> Self {
        Self::new()
    }
}
