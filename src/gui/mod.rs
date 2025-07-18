pub mod geocoding_panel;
pub mod layers_panel;
pub mod map_view;
pub mod style_editor;
pub mod tool_panel;
pub mod toolbar;
pub mod widgets;

use serde::{Deserialize, Serialize};

/// Geocoding result from a search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeResult {
    pub display_name: String,
    pub lat: f64,
    pub lon: f64,
    pub place_type: String,
    pub importance: f32,
}

/// GUI state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiState {
    pub show_style_editor_modal: bool,
    pub show_tool_panel: bool,
    pub show_about: bool,
    pub show_layers_panel: bool,
    pub show_geocoding_panel: bool,
    pub current_tool: Tool,
    pub zoom_level: f32,
    pub pan_offset: (f32, f32),
    pub selected_style: String,
    pub show_pois: bool,
    pub show_buildings: bool,
    pub show_roads: bool,
    pub show_water: bool,
    pub show_landuse: bool,
    pub show_gpx: bool,
    pub show_all_road_names: bool,
    pub search_query: String,
    pub geocoding_results: Vec<GeocodeResult>,
    pub is_geocoding: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Tool {
    Pan,
    Select,
    RectangleZoom,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            show_style_editor_modal: false,
            show_tool_panel: false,
            show_about: false,
            show_layers_panel: false,
            show_geocoding_panel: false,
            current_tool: Tool::Pan,
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
            selected_style: "google-maps".to_string(),
            show_pois: true,
            show_buildings: true,
            show_roads: true,
            show_water: true,
            show_landuse: true,
            show_gpx: false,
            show_all_road_names: false,
            search_query: String::new(),
            geocoding_results: Vec::new(),
            is_geocoding: false,
        }
    }
}

impl Default for GuiState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export GUI components
pub use geocoding_panel::{GeocodingPanel, GeocodingAction};
pub use layers_panel::LayersPanel;
pub use map_view::MapView;
pub use style_editor::StyleEditor;
pub use tool_panel::{ToolPanel, ToolPanelAction};
pub use toolbar::{Toolbar, ToolbarAction};
