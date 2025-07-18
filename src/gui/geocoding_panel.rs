use crate::gui::GuiState;
use crate::utils::geocoding::GeocodingService;
use egui::{Context, Window, ScrollArea, RichText, Color32};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Geocoding panel for searching addresses and places
pub struct GeocodingPanel {
    geocoding_service: Arc<Mutex<GeocodingService>>,
    runtime: tokio::runtime::Runtime,
}

/// Actions that can be triggered from the geocoding panel
#[derive(Debug, Clone)]
pub enum GeocodingAction {
    CenterOnLocation(f64, f64), // lat, lon
    None,
}

impl GeocodingPanel {
    pub fn new() -> Self {
        Self {
            geocoding_service: Arc::new(Mutex::new(GeocodingService::new())),
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
    
    pub fn show(&mut self, ctx: &Context, gui_state: &mut GuiState) -> GeocodingAction {
        if !gui_state.show_geocoding_panel {
            return GeocodingAction::None;
        }
        
        let mut open = true;
        let mut action = GeocodingAction::None;
        
        Window::new("🔍 Search Places")
            .open(&mut open)
            .resizable(true)
            .default_width(350.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    
                    // Search input
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        let search_input = ui.text_edit_singleline(&mut gui_state.search_query);
                        
                        // Search button
                        let search_button = ui.button("🔍 Search");
                        
                        // Trigger search on Enter key or button click
                        if (search_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                            || search_button.clicked() {
                            if !gui_state.search_query.trim().is_empty() {
                                self.perform_search(gui_state);
                            }
                        }
                    });
                    
                    // Loading indicator
                    if gui_state.is_geocoding {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Searching...");
                        });
                    }
                    
                    ui.separator();
                    
                    // Results
                    if !gui_state.geocoding_results.is_empty() {
                        ui.label(format!("Found {} results:", gui_state.geocoding_results.len()));
                        
                        ScrollArea::vertical()
                            .max_height(250.0)
                            .show(ui, |ui| {
                                for (i, result) in gui_state.geocoding_results.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.vertical(|ui| {
                                            // Place name and type
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(&result.display_name).strong());
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(RichText::new(&result.place_type).small().color(Color32::GRAY));
                                                });
                                            });
                                            
                                            // Coordinates
                                            ui.label(RichText::new(format!("📍 {:.6}, {:.6}", result.lat, result.lon)).small());
                                            
                                            // Go to location button
                                            if ui.button("📍 Go to location").clicked() {
                                                action = GeocodingAction::CenterOnLocation(result.lat, result.lon);
                                            }
                                        });
                                    });
                                    
                                    if i < gui_state.geocoding_results.len() - 1 {
                                        ui.separator();
                                    }
                                }
                            });
                    } else if !gui_state.is_geocoding && !gui_state.search_query.trim().is_empty() {
                        ui.colored_label(Color32::from_rgb(200, 200, 200), "No results found. Try a different search term.");
                    }
                    
                    if gui_state.geocoding_results.is_empty() && gui_state.search_query.trim().is_empty() {
                        ui.separator();
                        ui.colored_label(Color32::from_rgb(150, 150, 150), "💡 Examples:");
                        ui.colored_label(Color32::from_rgb(150, 150, 150), "• Paris, France");
                        ui.colored_label(Color32::from_rgb(150, 150, 150), "• 123 Main Street, New York");
                        ui.colored_label(Color32::from_rgb(150, 150, 150), "• Big Ben, London");
                        ui.colored_label(Color32::from_rgb(150, 150, 150), "• Central Park");
                    }
                });
            });
        
        // Update the gui_state if the window was closed
        if !open {
            gui_state.show_geocoding_panel = false;
        }
        
        action
    }
    
    fn perform_search(&mut self, gui_state: &mut GuiState) {
        if gui_state.is_geocoding {
            return; // Already searching
        }
        
        let query = gui_state.search_query.clone();
        
        gui_state.is_geocoding = true;
        gui_state.geocoding_results.clear();
        
        // Use a blocking approach to keep it simple
        let geocoding_service = Arc::clone(&self.geocoding_service);
        
        match self.runtime.block_on(async {
            let service = geocoding_service.lock().await;
            service.search(&query).await
        }) {
            Ok(results) => {
                gui_state.geocoding_results = results;
            }
            Err(e) => {
                eprintln!("Geocoding error: {}", e);
                gui_state.geocoding_results.clear();
            }
        }
        
        gui_state.is_geocoding = false;
    }
}

impl Default for GeocodingPanel {
    fn default() -> Self {
        Self::new()
    }
}
