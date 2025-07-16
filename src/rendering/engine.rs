use crate::rendering::{StyledMap, RenderedMap, RenderElement, ElementStyle, FeatureGeometry};
use crate::core::geometry::Transform2D;
use crate::export::ExportOptions;
use anyhow::Result;

pub struct RenderingEngine {
    // Advanced rendering capabilities
    pub enable_culling: bool,
    pub enable_simplification: bool,
    pub simplification_tolerance: f64,
    pub max_zoom_level: u8,
}

impl RenderingEngine {
    pub fn new() -> Self {
        Self {
            enable_culling: true,
            enable_simplification: true,
            simplification_tolerance: 0.5,
            max_zoom_level: 18,
        }
    }

    pub fn with_culling(mut self, enabled: bool) -> Self {
        self.enable_culling = enabled;
        self
    }

    pub fn with_simplification(mut self, enabled: bool, tolerance: f64) -> Self {
        self.enable_simplification = enabled;
        self.simplification_tolerance = tolerance;
        self
    }
    
    /// Render map with advanced features like culling and simplification
    pub fn render_advanced(&self, styled_map: &StyledMap, options: &ExportOptions) -> Result<RenderedMap> {
        let mut elements = Vec::new();
        
        // Calculate viewport bounds for culling
        let viewport_bounds = if self.enable_culling {
            Some(self.calculate_viewport_bounds(options))
        } else {
            None
        };
        
        // Calculate the transformation from geographic coordinates to screen coordinates
        let transform = self.calculate_transform(&styled_map.bounds, options.width, options.height);
        
        // Sort features by z-index for proper rendering order
        let mut sorted_features = styled_map.features.clone();
        sorted_features.sort_by_key(|f| f.z_index);
        
        // Render features with advanced optimizations
        for feature in sorted_features {
            // Skip features outside viewport if culling is enabled
            if self.enable_culling {
                if let Some(ref bounds) = viewport_bounds {
                    if !self.feature_intersects_viewport(&feature.geometry, bounds) {
                        continue;
                    }
                }
            }
            
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
                    let mut screen_coords: Vec<(f64, f64)> = coords
                        .iter()
                        .map(|coord| {
                            let screen_pos = transform.transform_point(coord);
                            (screen_pos.x, screen_pos.y)
                        })
                        .collect();
                    
                    // Apply line simplification if enabled
                    if self.enable_simplification && screen_coords.len() > 3 {
                        screen_coords = self.simplify_line(&screen_coords, self.simplification_tolerance);
                    }
                    
                    if screen_coords.len() >= 2 {
                        elements.push(RenderElement::Line {
                            points: screen_coords,
                            style: ElementStyle::from_render_style(&feature.style),
                        });
                    }
                }
                FeatureGeometry::Polygon { exterior, holes } => {
                    let mut screen_exterior: Vec<(f64, f64)> = exterior
                        .iter()
                        .map(|coord| {
                            let screen_pos = transform.transform_point(coord);
                            (screen_pos.x, screen_pos.y)
                        })
                        .collect();
                    
                    // Apply polygon simplification if enabled
                    if self.enable_simplification && screen_exterior.len() > 4 {
                        screen_exterior = self.simplify_polygon(&screen_exterior, self.simplification_tolerance);
                    }
                    
                    let screen_holes: Vec<Vec<(f64, f64)>> = holes
                        .iter()
                        .map(|hole| {
                            let mut screen_hole: Vec<(f64, f64)> = hole.iter()
                                .map(|coord| {
                                    let screen_pos = transform.transform_point(coord);
                                    (screen_pos.x, screen_pos.y)
                                })
                                .collect();
                            
                            if self.enable_simplification && screen_hole.len() > 4 {
                                screen_hole = self.simplify_polygon(&screen_hole, self.simplification_tolerance);
                            }
                            
                            screen_hole
                        })
                        .collect();
                    
                    if screen_exterior.len() >= 3 {
                        elements.push(RenderElement::Polygon {
                            exterior: screen_exterior,
                            holes: screen_holes,
                            style: ElementStyle::from_render_style(&feature.style),
                        });
                    }
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

    fn calculate_transform(&self, bounds: &crate::rendering::MapBounds, width: u32, height: u32) -> Transform2D {
        let map_width = bounds.max_lon - bounds.min_lon;
        let map_height = bounds.max_lat - bounds.min_lat;
        
        // Calculate scale to fit the map in the specified dimensions with padding
        let padding = 0.1; // 10% padding
        let scale_x = (width as f64 * (1.0 - padding)) / map_width;
        let scale_y = (height as f64 * (1.0 - padding)) / map_height;
        let scale = scale_x.min(scale_y);
        
        // Calculate translation to center the map
        let center_x = (bounds.min_lon + bounds.max_lon) / 2.0;
        let center_y = (bounds.min_lat + bounds.max_lat) / 2.0;
        
        let translate_x = width as f64 / 2.0 - center_x * scale;
        let translate_y = height as f64 / 2.0 - center_y * scale;
        
        Transform2D::translation(translate_x, translate_y)
            .compose(&Transform2D::scale(scale, -scale)) // Flip Y axis for screen coordinates
    }

    fn calculate_viewport_bounds(&self, options: &ExportOptions) -> ViewportBounds {
        // For now, use the full canvas as viewport
        ViewportBounds {
            min_x: 0.0,
            max_x: options.width as f64,
            min_y: 0.0,
            max_y: options.height as f64,
        }
    }

    fn feature_intersects_viewport(&self, _geometry: &FeatureGeometry, _bounds: &ViewportBounds) -> bool {
        // For now, always render (culling can be implemented more sophisticated)
        true
    }

    // Douglas-Peucker line simplification algorithm
    fn simplify_line(&self, points: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
        if points.len() <= 2 {
            return points.to_vec();
        }

        let mut simplified = Vec::new();
        self.douglas_peucker(points, tolerance, &mut simplified);
        
        if simplified.len() < 2 {
            // Fallback to original if simplification failed
            points.to_vec()
        } else {
            simplified
        }
    }

    fn simplify_polygon(&self, points: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
        if points.len() <= 4 {
            return points.to_vec();
        }

        // For polygons, we need to maintain closure
        let mut open_points = points.to_vec();
        if let (Some(first), Some(last)) = (open_points.first(), open_points.last()) {
            if (first.0 - last.0).abs() < 0.001 && (first.1 - last.1).abs() < 0.001 {
                open_points.pop(); // Remove duplicate closing point
            }
        }

        let mut simplified = Vec::new();
        self.douglas_peucker(&open_points, tolerance, &mut simplified);
        
        // Re-close the polygon
        if let Some(first) = simplified.first() {
            simplified.push(*first);
        }
        
        if simplified.len() < 4 {
            // Fallback to original if simplification failed
            points.to_vec()
        } else {
            simplified
        }
    }

    fn douglas_peucker(&self, points: &[(f64, f64)], tolerance: f64, result: &mut Vec<(f64, f64)>) {
        if points.len() <= 2 {
            result.extend_from_slice(points);
            return;
        }

        let first = points[0];
        let last = points[points.len() - 1];
        
        // Find the point with maximum distance from the line segment
        let mut max_distance = 0.0;
        let mut max_index = 0;
        
        for (i, &point) in points.iter().enumerate().skip(1).take(points.len() - 2) {
            let distance = self.point_to_line_distance(point, first, last);
            if distance > max_distance {
                max_distance = distance;
                max_index = i;
            }
        }
        
        // If the maximum distance is greater than tolerance, recursively simplify
        if max_distance > tolerance {
            let mut left_result = Vec::new();
            let mut right_result = Vec::new();
            
            self.douglas_peucker(&points[0..=max_index], tolerance, &mut left_result);
            self.douglas_peucker(&points[max_index..], tolerance, &mut right_result);
            
            result.extend_from_slice(&left_result[..left_result.len() - 1]);
            result.extend_from_slice(&right_result);
        } else {
            result.push(first);
            result.push(last);
        }
    }

    fn point_to_line_distance(&self, point: (f64, f64), line_start: (f64, f64), line_end: (f64, f64)) -> f64 {
        let a = line_end.1 - line_start.1;
        let b = line_start.0 - line_end.0;
        let c = line_end.0 * line_start.1 - line_start.0 * line_end.1;
        
        let distance = (a * point.0 + b * point.1 + c).abs() / (a * a + b * b).sqrt();
        distance
    }
}

#[derive(Debug, Clone)]
struct ViewportBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl Default for RenderingEngine {
    fn default() -> Self {
        Self::new()
    }
}
