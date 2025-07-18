use crate::gui::GuiState;
use egui::{Context, Window};

/// Floating layers panel for controlling map layer visibility
pub struct LayersPanel {
    pub is_open: bool,
}

impl LayersPanel {
    pub fn new() -> Self {
        Self {
            is_open: false,
        }
    }
    
    pub fn show(&mut self, ctx: &Context, gui_state: &mut GuiState) {
        if !gui_state.show_layers_panel {
            return;
        }
        
        let mut open = true;
        
        Window::new("🗺 Layers")
            .open(&mut open)
            .resizable(true)
            .default_width(250.0)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    
                    // Base Map Layer (always visible)
                    ui.horizontal(|ui| {
                        ui.add_enabled(false, egui::Checkbox::new(&mut true, ""));
                        ui.label("🗺 Base Map");
                    });
                    
                    ui.separator();
                    
                    // Buildings Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_buildings, "");
                        ui.label("🏠 Buildings");
                    });
                    
                    // Roads Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_roads, "");
                        ui.label("🛣 Roads");
                    });
                    
                    // All Road Names Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_all_road_names, "");
                        ui.label("📝 All Road Names");
                    });
                    
                    // Water Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_water, "");
                        ui.label("💧 Water");
                    });
                    
                    // Land Use Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_landuse, "");
                        ui.label("🌳 Land Use");
                    });
                    
                    // POIs Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_pois, "");
                        ui.label("📍 Points of Interest");
                    });
                    
                    // GPX Layer
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut gui_state.show_gpx, "");
                        ui.label("🚶 GPX Tracks");
                    });
                    
                    ui.separator();
                    
                    // Quick Actions
                    ui.horizontal(|ui| {
                        if ui.button("Show All").clicked() {
                            gui_state.show_buildings = true;
                            gui_state.show_roads = true;
                            gui_state.show_water = true;
                            gui_state.show_landuse = true;
                            gui_state.show_pois = true;
                            gui_state.show_gpx = true;
                            gui_state.show_all_road_names = true;
                        }
                        
                        if ui.button("Hide All").clicked() {
                            gui_state.show_buildings = false;
                            gui_state.show_roads = false;
                            gui_state.show_water = false;
                            gui_state.show_landuse = false;
                            gui_state.show_pois = false;
                            gui_state.show_gpx = false;
                            gui_state.show_all_road_names = false;
                        }
                    });
                    
                    ui.separator();
                    
                    // Layer info
                    ui.small("💡 Tip: Drag the window title to reposition");
                });
            });
        
        // Update the gui_state if the window was closed
        if !open {
            gui_state.show_layers_panel = false;
        }
    }
}

impl Default for LayersPanel {
    fn default() -> Self {
        Self::new()
    }
}
