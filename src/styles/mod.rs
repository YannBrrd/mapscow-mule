pub mod manager;

use crate::core::MapData;
use crate::parsers::stylesheet::{StyleRule, FeatureSelector, ElementType as StyleElementType, RenderStyle};
use crate::rendering::{StyledMap, StyledFeature, FeatureGeometry, MapBounds};
use anyhow::Result;
use std::collections::HashMap;

// Re-export for public API
pub use crate::parsers::stylesheet::StyleSheet;

/// Main style manager
pub struct StyleManager {
    stylesheets: Vec<StyleSheet>,
    active_stylesheet: Option<usize>,
}

impl StyleManager {
    pub fn new() -> Self {
        Self {
            stylesheets: vec![Self::create_default_stylesheet()],
            active_stylesheet: Some(0),
        }
    }
    
    pub fn add_stylesheet(&mut self, stylesheet: StyleSheet) -> usize {
        self.stylesheets.push(stylesheet);
        self.stylesheets.len() - 1
    }
    
    pub fn set_active_stylesheet(&mut self, index: usize) {
        if index < self.stylesheets.len() {
            self.active_stylesheet = Some(index);
        }
    }
    
    pub fn get_active_stylesheet(&self) -> Option<&StyleSheet> {
        self.active_stylesheet
            .and_then(|idx| self.stylesheets.get(idx))
    }
    
    pub fn get_active_stylesheet_mut(&mut self) -> Option<&mut StyleSheet> {
        self.active_stylesheet
            .and_then(|idx| self.stylesheets.get_mut(idx))
    }
    
    /// Apply styles to map data to create a styled map
    pub fn apply_styles(&self, map_data: &MapData) -> Result<StyledMap> {
        let stylesheet = self.get_active_stylesheet()
            .ok_or_else(|| anyhow::anyhow!("No active stylesheet"))?;
        
        let mut features = Vec::new();
        
        // Process ways (both lines and polygons)
        for way in map_data.ways.values() {
            if let Some(geometry) = map_data.get_way_geometry(way) {
                let feature_geometry = match geometry {
                    geo_types::Geometry::LineString(linestring) => {
                        FeatureGeometry::LineString(linestring.coords().cloned().collect())
                    }
                    geo_types::Geometry::Polygon(polygon) => {
                        let exterior: Vec<_> = polygon.exterior().coords().cloned().collect();
                        let holes: Vec<Vec<_>> = polygon.interiors()
                            .iter()
                            .map(|hole| hole.coords().cloned().collect())
                            .collect();
                        
                        FeatureGeometry::Polygon { exterior, holes }
                    }
                    _ => continue, // Skip other geometry types for now
                };
                
                // Find matching style rules
                for rule in &stylesheet.rules {
                    if self.matches_rule(rule, &way.tags, &StyleElementType::Way) {
                        let style = rule.style.clone();
                        let text = self.extract_text(&way.tags, &style);
                        
                        features.push(StyledFeature {
                            geometry: feature_geometry.clone(),
                            style,
                            text,
                            z_index: self.calculate_z_index(&way.tags),
                        });
                        break; // Use first matching rule
                    }
                }
            }
        }
        
        // Process nodes (points)
        for node in map_data.nodes.values() {
            if !node.tags.is_empty() { // Only process nodes with tags
                let feature_geometry = FeatureGeometry::Point(geo_types::Coord {
                    x: node.lon,
                    y: node.lat,
                });
                
                // Find matching style rules
                for rule in &stylesheet.rules {
                    if self.matches_rule(rule, &node.tags, &StyleElementType::Node) {
                        let style = rule.style.clone();
                        let text = self.extract_text(&node.tags, &style);
                        
                        features.push(StyledFeature {
                            geometry: feature_geometry.clone(),
                            style,
                            text,
                            z_index: self.calculate_z_index(&node.tags),
                        });
                        break; // Use first matching rule
                    }
                }
            }
        }
        
        // Sort features by z-index for proper rendering order
        features.sort_by_key(|f| f.z_index);
        
        let bounds = MapBounds {
            min_lat: map_data.bounds.min_lat,
            max_lat: map_data.bounds.max_lat,
            min_lon: map_data.bounds.min_lon,
            max_lon: map_data.bounds.max_lon,
        };
        
        Ok(StyledMap { features, bounds })
    }
    
    fn matches_rule(&self, rule: &StyleRule, tags: &HashMap<String, String>, element_type: &StyleElementType) -> bool {
        for selector in &rule.selectors {
            match selector {
                FeatureSelector::Tag { key, value } => {
                    if let Some(tag_value) = tags.get(key) {
                        match value {
                            Some(expected_value) => {
                                if tag_value == expected_value {
                                    return true;
                                }
                            }
                            None => return true, // Any value for this key
                        }
                    }
                }
                FeatureSelector::ElementType(selector_type) => {
                    if std::mem::discriminant(selector_type) == std::mem::discriminant(element_type) {
                        return true;
                    }
                }
                FeatureSelector::ZoomRange { min: _, max: _ } => {
                    // TODO: Implement zoom-based filtering
                    return true;
                }
            }
        }
        false
    }
    
    fn extract_text(&self, tags: &HashMap<String, String>, style: &RenderStyle) -> Option<String> {
        if let Some(ref text_field) = style.text_field {
            // Support simple tag references like "name" or more complex expressions
            if let Some(value) = tags.get(text_field) {
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn calculate_z_index(&self, tags: &HashMap<String, String>) -> i32 {
        // Calculate rendering order based on feature type
        if tags.contains_key("building") {
            100
        } else if tags.contains_key("landuse") {
            50
        } else if tags.contains_key("highway") {
            match tags.get("highway").map(|s| s.as_str()) {
                Some("motorway") | Some("trunk") => 80,
                Some("primary") => 70,
                Some("secondary") => 60,
                Some("tertiary") => 50,
                _ => 40,
            }
        } else if tags.contains_key("waterway") {
            30
        } else if tags.contains_key("natural") {
            20
        } else if tags.contains_key("amenity") {
            200 // Points should be on top
        } else {
            0
        }
    }
    
    fn create_default_stylesheet() -> StyleSheet {
        let mut stylesheet = StyleSheet::default();
        
        // Google Maps-inspired default stylesheet
        stylesheet.rules = vec![
            // Water bodies - light blue
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "natural".to_string(),
                        value: Some("water".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(170, 211, 223, 255)),
                    line_color: Some(crate::parsers::stylesheet::Color::new(140, 181, 193, 255)),
                    line_width: 1.0,
                    ..Default::default()
                },
            },
            
            // Waterways - rivers, streams
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "waterway".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(170, 211, 223, 255)),
                    line_width: 2.0,
                    ..Default::default()
                },
            },
            
            // Buildings - light gray with darker outlines
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "building".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Both,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(218, 218, 218, 255)),
                    line_color: Some(crate::parsers::stylesheet::Color::new(180, 180, 180, 255)),
                    line_width: 0.5,
                    ..Default::default()
                },
            },
            
            // Motorways - bold orange/red
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("motorway".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(231, 114, 0, 255)),
                    line_width: 6.0,
                    ..Default::default()
                },
            },
            
            // Trunk roads - orange
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("trunk".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(255, 156, 0, 255)),
                    line_width: 5.0,
                    ..Default::default()
                },
            },
            
            // Primary roads - yellow
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("primary".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(255, 205, 0, 255)),
                    line_width: 4.0,
                    ..Default::default()
                },
            },
            
            // Secondary roads - light yellow
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("secondary".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(255, 230, 100, 255)),
                    line_width: 3.5,
                    ..Default::default()
                },
            },
            
            // Tertiary roads - pale yellow
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("tertiary".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(255, 245, 150, 255)),
                    line_width: 3.0,
                    ..Default::default()
                },
            },
            
            // Residential roads - white
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("residential".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(255, 255, 255, 255)),
                    line_width: 2.5,
                    ..Default::default()
                },
            },
            
            // Service roads - light gray
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("service".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(240, 240, 240, 255)),
                    line_width: 1.5,
                    ..Default::default()
                },
            },
            
            // Footways and paths - dashed gray
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: Some("footway".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(200, 200, 200, 255)),
                    line_width: 1.0,
                    ..Default::default()
                },
            },
            
            // Parks and green spaces - light green
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "leisure".to_string(),
                        value: Some("park".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(194, 235, 164, 255)),
                    line_color: Some(crate::parsers::stylesheet::Color::new(174, 215, 144, 255)),
                    line_width: 0.5,
                    ..Default::default()
                },
            },
            
            // Forests - darker green
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "natural".to_string(),
                        value: Some("wood".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(173, 209, 158, 255)),
                    ..Default::default()
                },
            },
            
            // Landuse - forest
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "landuse".to_string(),
                        value: Some("forest".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(173, 209, 158, 255)),
                    ..Default::default()
                },
            },
            
            // Grass areas - light green
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "landuse".to_string(),
                        value: Some("grass".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(194, 235, 164, 255)),
                    ..Default::default()
                },
            },
            
            // Commercial areas - light pink
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "landuse".to_string(),
                        value: Some("commercial".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(255, 230, 230, 255)),
                    ..Default::default()
                },
            },
            
            // Industrial areas - light purple
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "landuse".to_string(),
                        value: Some("industrial".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(230, 220, 240, 255)),
                    ..Default::default()
                },
            },
            
            // Residential areas - very light yellow
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "landuse".to_string(),
                        value: Some("residential".to_string()),
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Fill,
                    fill_color: Some(crate::parsers::stylesheet::Color::new(255, 255, 230, 255)),
                    ..Default::default()
                },
            },
            
            // Railway lines - dark gray
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "railway".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(100, 100, 100, 255)),
                    line_width: 2.0,
                    ..Default::default()
                },
            },
            
            // POI - Amenities (restaurants, shops, etc.) - red circles
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "amenity".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Point,
                    line_color: Some(crate::parsers::stylesheet::Color::new(220, 20, 60, 255)),
                    line_width: 4.0,
                    ..Default::default()
                },
            },
            
            // Shops - blue circles
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "shop".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Point,
                    line_color: Some(crate::parsers::stylesheet::Color::new(30, 144, 255, 255)),
                    line_width: 3.5,
                    ..Default::default()
                },
            },
            
            // Tourism - green circles
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "tourism".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Point,
                    line_color: Some(crate::parsers::stylesheet::Color::new(50, 205, 50, 255)),
                    line_width: 3.5,
                    ..Default::default()
                },
            },
            
            // Default fallback for any unmatched highways
            StyleRule {
                selectors: vec![
                    FeatureSelector::Tag {
                        key: "highway".to_string(),
                        value: None,
                    }
                ],
                style: RenderStyle {
                    draw_mode: crate::parsers::stylesheet::DrawMode::Line,
                    line_color: Some(crate::parsers::stylesheet::Color::new(200, 200, 200, 255)),
                    line_width: 1.5,
                    ..Default::default()
                },
            },
        ];
        
        stylesheet
    }
}

impl Default for StyleManager {
    fn default() -> Self {
        Self::new()
    }
}
