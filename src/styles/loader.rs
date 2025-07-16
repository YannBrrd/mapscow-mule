use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapStyle {
    pub background: BackgroundStyle,
    pub water: WaterStyle,
    pub landuse: HashMap<String, String>,
    pub leisure: HashMap<String, String>,
    pub natural: HashMap<String, String>,
    pub aeroway: AerowayStyle,
    pub buildings: BuildingStyle,
    pub roads: HashMap<String, RoadStyle>,
    pub railway: RailwayStyle,
    pub boundaries: BoundaryStyle,
    pub pois: HashMap<String, PoiStyle>,
    pub labels: LabelStyle,
    pub road_label_fonts: HashMap<String, u32>,
    pub place_label_fonts: HashMap<String, u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackgroundStyle {
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WaterStyle {
    pub color: String,
    pub opacity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AerowayStyle {
    pub default: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildingStyle {
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoadStyle {
    pub color: String,
    pub width: f32,
    pub border_color: String,
    pub border_width: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RailwayStyle {
    pub rail_color: String,
    pub rail_width: f32,
    pub rail_dash_color: String,
    pub rail_dash_width: f32,
    pub rail_dash_pattern: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoundaryStyle {
    pub administrative_color: String,
    pub administrative_width: f32,
    pub administrative_dash: String,
    pub administrative_opacity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoiStyle {
    pub color: String,
    pub radius: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LabelStyle {
    pub font_family: String,
    pub road_label_stroke: String,
    pub road_label_stroke_width: u32,
    pub poi_label_stroke: String,
    pub poi_label_stroke_width: u32,
    pub place_label_stroke: String,
    pub place_label_stroke_width: u32,
}

impl MapStyle {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let style: MapStyle = toml::from_str(&content)?;
        Ok(style)
    }
    
    pub fn load_google_maps() -> Result<Self> {
        let style_path = Path::new("assets/styles/google-maps.toml");
        Self::load_from_file(style_path)
    }
    
    pub fn load_osm_default() -> Result<Self> {
        let style_path = Path::new("assets/styles/osm-default.toml");
        Self::load_from_file(style_path)
    }
    
    pub fn load_modern_clean() -> Result<Self> {
        let style_path = Path::new("assets/styles/modern-clean.toml");
        Self::load_from_file(style_path)
    }
    
    pub fn get_road_style(&self, highway: &str) -> (&str, f32, &str, f32) {
        if let Some(style) = self.roads.get(highway) {
            (&style.color, style.width, &style.border_color, style.border_width)
        } else {
            // Default road style
            ("#e0e0e0", 1.0, "", 0.0)
        }
    }
    
    pub fn get_poi_style(&self, amenity: &str) -> (&str, f32) {
        if let Some(style) = self.pois.get(amenity) {
            (&style.color, style.radius)
        } else if let Some(default_style) = self.pois.get("default") {
            (&default_style.color, default_style.radius)
        } else {
            ("#95a5a6", 2.5)
        }
    }
    
    pub fn get_landuse_color(&self, landuse: &str) -> Option<&str> {
        self.landuse.get(landuse).map(|s| s.as_str())
    }
    
    pub fn get_leisure_color(&self, leisure: &str) -> Option<&str> {
        self.leisure.get(leisure).map(|s| s.as_str())
    }
    
    pub fn get_natural_color(&self, natural: &str) -> Option<&str> {
        self.natural.get(natural).map(|s| s.as_str())
    }
    
    pub fn get_road_label_font_size(&self, highway: &str) -> u32 {
        self.road_label_fonts.get(highway)
            .copied()
            .unwrap_or(8)
    }
    
    pub fn get_place_label_font_size(&self, place: &str) -> u32 {
        self.place_label_fonts.get(place)
            .or_else(|| self.place_label_fonts.get("default"))
            .copied()
            .unwrap_or(9)
    }
}

pub struct StyleManager {
    current_style: MapStyle,
    available_styles: HashMap<String, PathBuf>,
}

impl StyleManager {
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            current_style: MapStyle::load_google_maps()?,
            available_styles: HashMap::new(),
        };
        
        manager.scan_available_styles()?;
        Ok(manager)
    }
    
    pub fn new_with_default() -> Result<Self> {
        // Fallback to google-maps style without scanning directory
        Ok(Self {
            current_style: MapStyle::load_google_maps()?,
            available_styles: {
                let mut styles = HashMap::new();
                styles.insert("google-maps".to_string(), PathBuf::from("assets/styles/google-maps.toml"));
                styles.insert("osm-default".to_string(), PathBuf::from("assets/styles/osm-default.toml"));
                styles
            },
        })
    }
    
    pub fn scan_available_styles(&mut self) -> Result<()> {
        let styles_dir = Path::new("assets/styles");
        if styles_dir.exists() {
            for entry in std::fs::read_dir(styles_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        self.available_styles.insert(stem.to_string(), path);
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn load_style(&mut self, style_name: &str) -> Result<()> {
        if let Some(path) = self.available_styles.get(style_name) {
            self.current_style = MapStyle::load_from_file(path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Style '{}' not found", style_name))
        }
    }
    
    pub fn get_current_style(&self) -> &MapStyle {
        &self.current_style
    }
    
    pub fn get_available_styles(&self) -> Vec<&str> {
        self.available_styles.keys().map(|s| s.as_str()).collect()
    }
}
