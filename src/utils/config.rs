use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub map: MapConfig,
    pub export: ExportConfig,
    pub recent_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub show_style_editor: bool,
    pub show_tool_panel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub default_projection: String,
    pub background_color: (u8, u8, u8, u8),
    pub cache_enabled: bool,
    pub cache_size_mb: u32,
    pub default_style: String,  // Added default style preference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub default_format: String,
    pub default_dpi: f32,
    pub default_width: u32,
    pub default_height: u32,
    pub last_export_directory: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 1200,
                height: 800,
                maximized: false,
                show_style_editor: true,
                show_tool_panel: true,
            },
            map: MapConfig {
                default_projection: "WebMercator".to_string(),
                background_color: (240, 248, 255, 255),
                cache_enabled: true,
                cache_size_mb: 256,
                default_style: "google-maps".to_string(),  // Default to Google Maps style
            },
            export: ExportConfig {
                default_format: "svg".to_string(),
                default_dpi: 300.0,
                default_width: 1920,
                default_height: 1080,
                last_export_directory: None,
            },
            recent_files: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => {
                        match serde_yaml::from_str(&content) {
                            Ok(config) => return config,
                            Err(e) => log::warn!("Failed to parse config file: {}", e),
                        }
                    }
                    Err(e) => log::warn!("Failed to read config file: {}", e),
                }
            }
        }
        
        Self::default()
    }
    
    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            let content = serde_yaml::to_string(self)?;
            std::fs::write(&config_path, content)?;
        }
        
        Ok(())
    }
    
    /// Get the path to the configuration file
    fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("mapscow-mule");
            path.push("config.yaml");
            path
        })
    }
    
    /// Add a file to the recent files list
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        
        // Add to front
        self.recent_files.insert(0, path);
        
        // Keep only last 10
        self.recent_files.truncate(10);
    }
}
