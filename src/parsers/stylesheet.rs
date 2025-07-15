use crate::parsers::{Parser, ParseError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Stylesheet parser for map styling rules
pub struct StylesheetParser;

impl StylesheetParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser<StyleSheet> for StylesheetParser {
    fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<StyleSheet> {
        let content = std::fs::read_to_string(path)?;
        self.parse_string(&content)
    }
    
    fn parse_string(&self, content: &str) -> Result<StyleSheet> {
        // Try YAML first, then fallback to our custom format
        if let Ok(stylesheet) = serde_yaml::from_str::<StyleSheet>(content) {
            Ok(stylesheet)
        } else {
            self.parse_custom_format(content)
        }
    }
}

impl StylesheetParser {
    /// Parse custom Maperitive-style format
    fn parse_custom_format(&self, content: &str) -> Result<StyleSheet> {
        let mut stylesheet = StyleSheet::default();
        let mut current_rule: Option<StyleRule> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }
            
            // Parse rule definitions
            if line.starts_with("features") {
                if let Some(rule) = current_rule.take() {
                    stylesheet.rules.push(rule);
                }
                current_rule = Some(self.parse_features_line(line)?);
            } else if line.starts_with("define") {
                // Color/variable definitions
                self.parse_define_line(line, &mut stylesheet)?;
            } else if let Some(ref mut rule) = current_rule {
                // Style properties
                self.parse_style_property(line, rule)?;
            }
        }
        
        // Add the last rule
        if let Some(rule) = current_rule {
            stylesheet.rules.push(rule);
        }
        
        Ok(stylesheet)
    }
    
    fn parse_features_line(&self, line: &str) -> Result<StyleRule> {
        // Example: "features amenity=restaurant"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParseError::InvalidFormat("Invalid features line".to_string()).into());
        }
        
        let mut selectors = Vec::new();
        
        for selector_str in &parts[1..] {
            if selector_str.contains('=') {
                let (key, value) = selector_str.split_once('=').unwrap();
                selectors.push(FeatureSelector::Tag {
                    key: key.to_string(),
                    value: Some(value.to_string()),
                });
            } else {
                selectors.push(FeatureSelector::Tag {
                    key: selector_str.to_string(),
                    value: None,
                });
            }
        }
        
        Ok(StyleRule {
            selectors,
            style: RenderStyle::default(),
        })
    }
    
    fn parse_define_line(&self, line: &str, stylesheet: &mut StyleSheet) -> Result<()> {
        // Example: "define water-color #4A90E2"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "define" {
            let name = parts[1].to_string();
            let value = parts[2..].join(" ");
            
            if let Ok(color) = self.parse_color(&value) {
                stylesheet.variables.insert(name, StyleVariable::Color(color));
            } else {
                stylesheet.variables.insert(name, StyleVariable::String(value));
            }
        }
        
        Ok(())
    }
    
    fn parse_style_property(&self, line: &str, rule: &mut StyleRule) -> Result<()> {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "draw" => {
                    rule.style.draw_mode = match value {
                        "line" => DrawMode::Line,
                        "fill" => DrawMode::Fill,
                        "both" => DrawMode::Both,
                        "point" => DrawMode::Point,
                        _ => DrawMode::Line,
                    };
                }
                "line-color" => {
                    rule.style.line_color = Some(self.parse_color(value)?);
                }
                "fill-color" => {
                    rule.style.fill_color = Some(self.parse_color(value)?);
                }
                "line-width" => {
                    rule.style.line_width = value.parse::<f32>().unwrap_or(1.0);
                }
                "font-family" => {
                    rule.style.font_family = Some(value.to_string());
                }
                "font-size" => {
                    rule.style.font_size = value.parse::<f32>().unwrap_or(12.0);
                }
                "text" => {
                    rule.style.text_field = Some(value.to_string());
                }
                "min-zoom" => {
                    rule.style.min_zoom = Some(value.parse::<u32>().unwrap_or(0));
                }
                "max-zoom" => {
                    rule.style.max_zoom = Some(value.parse::<u32>().unwrap_or(18));
                }
                _ => {
                    // Unknown property - could log a warning
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_color(&self, color_str: &str) -> Result<Color> {
        let color_str = color_str.trim();
        
        // Hex color
        if color_str.starts_with('#') {
            let hex = &color_str[1..];
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16)?;
                let g = u8::from_str_radix(&hex[2..4], 16)?;
                let b = u8::from_str_radix(&hex[4..6], 16)?;
                return Ok(Color::new(r, g, b, 255));
            } else if hex.len() == 8 {
                let r = u8::from_str_radix(&hex[0..2], 16)?;
                let g = u8::from_str_radix(&hex[2..4], 16)?;
                let b = u8::from_str_radix(&hex[4..6], 16)?;
                let a = u8::from_str_radix(&hex[6..8], 16)?;
                return Ok(Color::new(r, g, b, a));
            }
        }
        
        // RGB/RGBA function
        if color_str.starts_with("rgb(") || color_str.starts_with("rgba(") {
            let inner = color_str
                .trim_start_matches("rgb(")
                .trim_start_matches("rgba(")
                .trim_end_matches(')');
            
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() >= 3 {
                let r = parts[0].trim().parse::<u8>()?;
                let g = parts[1].trim().parse::<u8>()?;
                let b = parts[2].trim().parse::<u8>()?;
                let a = if parts.len() > 3 {
                    (parts[3].trim().parse::<f32>()? * 255.0) as u8
                } else {
                    255
                };
                return Ok(Color::new(r, g, b, a));
            }
        }
        
        // Named colors
        match color_str.to_lowercase().as_str() {
            "red" => Ok(Color::new(255, 0, 0, 255)),
            "green" => Ok(Color::new(0, 255, 0, 255)),
            "blue" => Ok(Color::new(0, 0, 255, 255)),
            "white" => Ok(Color::new(255, 255, 255, 255)),
            "black" => Ok(Color::new(0, 0, 0, 255)),
            "transparent" => Ok(Color::new(0, 0, 0, 0)),
            _ => Err(ParseError::InvalidFormat(format!("Unknown color: {}", color_str)).into()),
        }
    }
}

/// Complete stylesheet with rules and variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
    pub variables: HashMap<String, StyleVariable>,
}

/// Individual styling rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    pub selectors: Vec<FeatureSelector>,
    pub style: RenderStyle,
}

/// Feature selector for matching map elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureSelector {
    Tag { key: String, value: Option<String> },
    ElementType(ElementType),
    ZoomRange { min: Option<u32>, max: Option<u32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Node,
    Way,
    Relation,
}

/// Rendering style properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderStyle {
    pub draw_mode: DrawMode,
    pub line_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub line_width: f32,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub text_field: Option<String>,
    pub min_zoom: Option<u32>,
    pub max_zoom: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DrawMode {
    Line,
    Fill,
    Both,
    Point,
    Text,
}

/// Color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn to_rgba_f32(&self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }
    
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

/// Variable types that can be defined in stylesheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleVariable {
    Color(Color),
    Number(f64),
    String(String),
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            variables: HashMap::new(),
        }
    }
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            draw_mode: DrawMode::Line,
            line_color: Some(Color::new(0, 0, 0, 255)),
            fill_color: None,
            line_width: 1.0,
            font_family: Some("Arial".to_string()),
            font_size: 12.0,
            text_field: None,
            min_zoom: None,
            max_zoom: None,
        }
    }
}

impl Default for StylesheetParser {
    fn default() -> Self {
        Self::new()
    }
}
