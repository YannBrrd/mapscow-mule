pub mod engine;

use crate::core::geometry::Transform2D;
use crate::export::ExportOptions;
use crate::parsers::stylesheet::{Color, RenderStyle};
use anyhow::Result;
use geo_types::Coord;

/// Main map renderer
pub struct MapRenderer {
    transform: Transform2D,
}

impl MapRenderer {
    pub fn new() -> Self {
        Self {
            transform: Transform2D::identity(),
        }
    }
    
    pub fn render(&self, styled_map: &StyledMap, options: &ExportOptions) -> Result<RenderedMap> {
        let mut elements = Vec::new();
        
        // Calculate the transformation from geographic coordinates to screen coordinates
        let transform = self.calculate_transform(&styled_map.bounds, options.width, options.height);
        
        // Render features in proper order (fills first, then lines, then points, then text)
        for feature in &styled_map.features {
            match &feature.geometry {
                FeatureGeometry::Point(coord) => {
                    let screen_pos = transform.transform_point(coord);
                    elements.push(RenderElement::Circle {
                        center: (screen_pos.x, screen_pos.y),
                        radius: feature.style.point_radius().unwrap_or(3.0) as f64,
                        style: ElementStyle::from_render_style(&feature.style),
                    });
                }
                FeatureGeometry::LineString(coords) => {
                    let screen_coords: Vec<(f64, f64)> = coords
                        .iter()
                        .map(|coord| {
                            let screen_pos = transform.transform_point(coord);
                            (screen_pos.x, screen_pos.y)
                        })
                        .collect();
                    
                    elements.push(RenderElement::Line {
                        points: screen_coords,
                        style: ElementStyle::from_render_style(&feature.style),
                    });
                }
                FeatureGeometry::Polygon { exterior, holes } => {
                    let screen_exterior: Vec<(f64, f64)> = exterior
                        .iter()
                        .map(|coord| {
                            let screen_pos = transform.transform_point(coord);
                            (screen_pos.x, screen_pos.y)
                        })
                        .collect();
                    
                    let screen_holes: Vec<Vec<(f64, f64)>> = holes
                        .iter()
                        .map(|hole| {
                            hole.iter()
                                .map(|coord| {
                                    let screen_pos = transform.transform_point(coord);
                                    (screen_pos.x, screen_pos.y)
                                })
                                .collect()
                        })
                        .collect();
                    
                    elements.push(RenderElement::Polygon {
                        exterior: screen_exterior,
                        holes: screen_holes,
                        style: ElementStyle::from_render_style(&feature.style),
                    });
                }
            }
            
            // Add text label if specified
            if let Some(ref text) = feature.text {
                if let Some(center) = feature.geometry.center() {
                    let screen_pos = transform.transform_point(&center);
                    elements.push(RenderElement::Text {
                        position: (screen_pos.x, screen_pos.y),
                        text: text.clone(),
                        style: ElementStyle::from_render_style(&feature.style),
                    });
                }
            }
        }
        
        Ok(RenderedMap { elements })
    }
    
    fn calculate_transform(&self, bounds: &MapBounds, width: u32, height: u32) -> Transform2D {
        let map_width = bounds.max_lon - bounds.min_lon;
        let map_height = bounds.max_lat - bounds.min_lat;
        
        // Calculate scale to fit the map in the specified dimensions
        let scale_x = width as f64 / map_width;
        let scale_y = height as f64 / map_height;
        let scale = scale_x.min(scale_y);
        
        // Calculate translation to center the map
        let center_x = (bounds.min_lon + bounds.max_lon) / 2.0;
        let center_y = (bounds.min_lat + bounds.max_lat) / 2.0;
        
        let translate_x = width as f64 / 2.0 - center_x * scale;
        let translate_y = height as f64 / 2.0 - center_y * scale;
        
        Transform2D::translation(translate_x, translate_y)
            .compose(&Transform2D::scale(scale, -scale)) // Flip Y axis for screen coordinates
    }
}

/// Map with applied styling
#[derive(Debug, Clone)]
pub struct StyledMap {
    pub features: Vec<StyledFeature>,
    pub bounds: MapBounds,
}

/// Individual feature with applied style
#[derive(Debug, Clone)]
pub struct StyledFeature {
    pub geometry: FeatureGeometry,
    pub style: RenderStyle,
    pub text: Option<String>,
    pub z_index: i32,
}

/// Simplified geometry types for rendering
#[derive(Debug, Clone)]
pub enum FeatureGeometry {
    Point(Coord<f64>),
    LineString(Vec<Coord<f64>>),
    Polygon {
        exterior: Vec<Coord<f64>>,
        holes: Vec<Vec<Coord<f64>>>,
    },
}

impl FeatureGeometry {
    pub fn center(&self) -> Option<Coord<f64>> {
        match self {
            FeatureGeometry::Point(coord) => Some(*coord),
            FeatureGeometry::LineString(coords) => {
                if coords.is_empty() {
                    None
                } else {
                    let sum_x: f64 = coords.iter().map(|c| c.x).sum();
                    let sum_y: f64 = coords.iter().map(|c| c.y).sum();
                    Some(Coord {
                        x: sum_x / coords.len() as f64,
                        y: sum_y / coords.len() as f64,
                    })
                }
            }
            FeatureGeometry::Polygon { exterior, .. } => {
                if exterior.is_empty() {
                    None
                } else {
                    let sum_x: f64 = exterior.iter().map(|c| c.x).sum();
                    let sum_y: f64 = exterior.iter().map(|c| c.y).sum();
                    Some(Coord {
                        x: sum_x / exterior.len() as f64,
                        y: sum_y / exterior.len() as f64,
                    })
                }
            }
        }
    }
}

/// Bounds for styled map
#[derive(Debug, Clone, Copy)]
pub struct MapBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// Final rendered map ready for export
#[derive(Debug, Clone)]
pub struct RenderedMap {
    pub elements: Vec<RenderElement>,
}

/// Individual render element
#[derive(Debug, Clone)]
pub enum RenderElement {
    Line {
        points: Vec<(f64, f64)>,
        style: ElementStyle,
    },
    Polygon {
        exterior: Vec<(f64, f64)>,
        holes: Vec<Vec<(f64, f64)>>,
        style: ElementStyle,
    },
    Circle {
        center: (f64, f64),
        radius: f64,
        style: ElementStyle,
    },
    Text {
        position: (f64, f64),
        text: String,
        style: ElementStyle,
    },
}

/// Rendering style for individual elements
#[derive(Debug, Clone)]
pub struct ElementStyle {
    pub stroke_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
    pub stroke_opacity: f32,
    pub fill_opacity: f32,
    pub stroke_dash: Vec<f32>,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: u32,
    pub point_radius: Option<f32>,
}

impl ElementStyle {
    pub fn from_render_style(style: &RenderStyle) -> Self {
        Self {
            stroke_color: style.line_color,
            fill_color: style.fill_color,
            stroke_width: style.line_width,
            stroke_opacity: 1.0,
            fill_opacity: 1.0,
            stroke_dash: Vec::new(),
            font_family: style.font_family.clone(),
            font_size: style.font_size,
            font_weight: 400,
            point_radius: None,
        }
    }
}

impl RenderStyle {
    pub fn point_radius(&self) -> Option<f32> {
        // Default point radius if not specified
        Some(3.0)
    }
}

impl Default for MapRenderer {
    fn default() -> Self {
        Self::new()
    }
}
