pub mod map_view;
pub mod style_editor;
pub mod tool_panel;
pub mod widgets;

use serde::{Deserialize, Serialize};

/// GUI state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiState {
    pub show_style_editor: bool,
    pub show_tool_panel: bool,
    pub show_about: bool,
    pub current_tool: Tool,
    pub zoom_level: f32,
    pub pan_offset: (f32, f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Tool {
    Pan,
    Zoom,
    Measure,
    Select,
    RectangleZoom,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            show_style_editor: true,
            show_tool_panel: true,
            show_about: false,
            current_tool: Tool::Pan,
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
        }
    }
}

impl Default for GuiState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export GUI components
pub use map_view::MapView;
pub use style_editor::StyleEditor;
pub use tool_panel::ToolPanel;
