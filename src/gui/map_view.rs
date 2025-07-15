use crate::core::MapData;
use crate::rendering::MapRenderer;
use crate::styles::StyleManager;
use egui::{Ui, Response, Sense, Vec2, Pos2, Rect, Color32};

/// Main map view widget
pub struct MapView {
    /// Last known mouse position for drag operations
    last_mouse_pos: Option<Pos2>,
    /// Viewport bounds in map coordinates
    viewport: Viewport,
    /// Rectangle selection for zoom-to-area
    selection_rect: Option<SelectionRect>,
    /// Whether rectangle selection mode is active
    selection_mode: bool,
}

#[derive(Debug, Clone)]
struct SelectionRect {
    start_pos: Pos2,
    current_pos: Pos2,
    has_been_dragged: bool,
}

#[derive(Debug, Clone)]
struct Viewport {
    center_x: f64,
    center_y: f64,
    scale: f64,
    width: f32,
    height: f32,
}

impl MapView {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: None,
            viewport: Viewport {
                center_x: 0.0,
                center_y: 0.0,
                scale: 1.0,
                width: 800.0,
                height: 600.0,
            },
            selection_rect: None,
            selection_mode: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, map_data: &Option<MapData>, renderer: &MapRenderer, style_manager: &StyleManager) -> Response {
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag().union(Sense::hover()));
        
        // Update viewport size
        self.viewport.width = rect.width();
        self.viewport.height = rect.height();
        
        // Handle input (pass the rect for coordinate conversion)
        self.handle_input(ui, &response, rect);
        
        // Draw the map
        self.draw_map(ui, rect, map_data, renderer, style_manager);
        
        response
    }
    
    /// Toggle rectangle selection mode on/off
    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = !self.selection_mode;
        self.selection_rect = None; // Clear any existing selection
        println!("Rectangle zoom selection mode: {}", if self.selection_mode { "ON" } else { "OFF" });
    }
    
    /// Check if selection mode is active
    pub fn is_selection_mode(&self) -> bool {
        self.selection_mode
    }
    
    /// Get current zoom level
    pub fn get_zoom_level(&self) -> f64 {
        self.viewport.scale
    }

    /// Zoom by a specific factor (e.g., 1.2 for zoom in, 0.83 for zoom out)
    pub fn zoom_by_factor(&mut self, factor: f64) {
        self.viewport.scale *= factor;
        self.viewport.scale = self.viewport.scale.clamp(0.001, 500000.0);
    }
    
    /// Get viewport information for export (center coordinates and scale)
    pub fn get_viewport_info(&self) -> (f64, f64, f64) {
        (self.viewport.center_x, self.viewport.center_y, self.viewport.scale)
    }

    fn handle_input(&mut self, ui: &mut Ui, response: &Response, rect: Rect) {
        // Handle mouse wheel for zooming FIRST (should work in all modes)
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        if scroll_delta.y != 0.0 {
            println!("Zoom event detected: scroll_delta.y = {}", scroll_delta.y);
            let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 1.0 / 1.1 };
            
            // Zoom towards mouse position if available
            if let Some(mouse_pos) = response.hover_pos() {
                let rel_x = (mouse_pos.x - rect.center().x) as f64;
                let rel_y = -(mouse_pos.y - rect.center().y) as f64; // Flip Y
                
                // Convert to map coordinates
                let map_x = self.viewport.center_x + rel_x / self.viewport.scale;
                let map_y = self.viewport.center_y + rel_y / self.viewport.scale;
                
                // Apply zoom
                let old_scale = self.viewport.scale;
                self.viewport.scale *= zoom_factor;
                println!("Zoom applied: {} -> {}", old_scale, self.viewport.scale);
                
                // Adjust center to zoom towards mouse position
                self.viewport.center_x = map_x - rel_x / self.viewport.scale;
                self.viewport.center_y = map_y - rel_y / self.viewport.scale;
            } else {
                // Simple zoom at center
                let old_scale = self.viewport.scale;
                self.viewport.scale *= zoom_factor;
                println!("Simple zoom applied: {} -> {}", old_scale, self.viewport.scale);
            }
            
            // Clamp zoom level to allow for very detailed viewing
            self.viewport.scale = self.viewport.scale.clamp(0.001, 500000.0);
        }

        // Handle rectangle selection mode
        if self.selection_mode {
            self.handle_selection_input(response, rect);
            return; // Skip normal panning/zooming in selection mode
        }
        
        // Handle mouse drag for panning
        if response.dragged() {
            if let Some(last_pos) = self.last_mouse_pos {
                if let Some(current_pos) = response.interact_pointer_pos() {
                    let delta = current_pos - last_pos;
                    
                    // Convert screen delta to map coordinates
                    let map_delta_x = (delta.x as f64) / self.viewport.scale;
                    let map_delta_y = -(delta.y as f64) / self.viewport.scale; // Flip Y axis
                    
                    self.viewport.center_x -= map_delta_x;
                    self.viewport.center_y -= map_delta_y;
                }
            }
            self.last_mouse_pos = response.interact_pointer_pos();
        } else {
            self.last_mouse_pos = None;
        }
    }
    
    fn draw_map(&self, ui: &mut Ui, rect: Rect, map_data: &Option<MapData>, renderer: &MapRenderer, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        // Draw Google Maps-style background (light beige/gray)
        painter.rect_filled(rect, 0.0, Color32::from_rgb(248, 246, 240));
        
        if let Some(data) = map_data {
            // Calculate visible bounds
            let visible_bounds = self.calculate_visible_bounds(rect);
            
            // Draw map features in proper order (like Google Maps)
            // 1. Water bodies and areas (lowest layer)
            self.draw_water_areas(ui, rect, data, &visible_bounds, style_manager);
            
            // 2. Land use areas (parks, forests, etc.)
            self.draw_landuse_areas(ui, rect, data, &visible_bounds, style_manager);
            
            // 3. Buildings (with shadows for 3D effect)
            self.draw_buildings(ui, rect, data, &visible_bounds, style_manager);
            
            // 4. Road casings (dark outlines first)
            self.draw_road_casings(ui, rect, data, &visible_bounds, style_manager);
            
            // 5. Road fills (lighter colors on top)
            self.draw_road_fills(ui, rect, data, &visible_bounds, style_manager);
            
            // 6. Railways and other transport
            self.draw_railways(ui, rect, data, &visible_bounds, style_manager);
            
            // 7. Points of interest
            self.draw_points_of_interest(ui, rect, data, &visible_bounds, style_manager);
            
            // 8. Text labels (highest layer)
            self.draw_text_labels(ui, rect, data, &visible_bounds, style_manager);
            
            // Debug info (smaller and less intrusive)
            if self.viewport.scale > 1000.0 { // Only show at high zoom
                let debug_text = format!(
                    "Zoom: {:.1}x | Features: {}/{}", 
                    self.viewport.scale,
                    data.ways.len(),
                    data.nodes.len()
                );
                painter.text(
                    rect.min + egui::Vec2::new(10.0, rect.height() - 25.0),
                    egui::Align2::LEFT_BOTTOM,
                    debug_text,
                    egui::FontId::monospace(9.0),
                    Color32::from_gray(120),
                );
            }
        } else {
            // Draw placeholder text
            let text = "No map data loaded";
            let text_color = Color32::from_rgb(128, 128, 128);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(16.0),
                text_color,
            );
        }
        
        // Draw selection rectangle if active
        self.draw_selection_rectangle(ui, rect);
        
        // Draw viewport info (cleaner style)
        self.draw_viewport_info(ui, rect);
    }
    
    fn map_to_screen(&self, lon: f64, lat: f64, rect: Rect) -> Pos2 {
        let x = (lon - self.viewport.center_x) * self.viewport.scale + (rect.width() / 2.0) as f64;
        let y = -(lat - self.viewport.center_y) * self.viewport.scale + (rect.height() / 2.0) as f64;
        
        Pos2::new(
            rect.min.x + x as f32,
            rect.min.y + y as f32,
        )
    }
    
    fn calculate_visible_bounds(&self, rect: Rect) -> VisibleBounds {
        let half_width = (rect.width() / 2.0) as f64 / self.viewport.scale;
        let half_height = (rect.height() / 2.0) as f64 / self.viewport.scale;
        
        VisibleBounds {
            min_lon: self.viewport.center_x - half_width,
            max_lon: self.viewport.center_x + half_width,
            min_lat: self.viewport.center_y - half_height,
            max_lat: self.viewport.center_y + half_height,
        }
    }
    
    fn way_intersects_bounds(&self, way: &crate::core::Way, map_data: &MapData, bounds: &VisibleBounds) -> bool {
        // Simple bounds check - in production, you'd want more sophisticated culling
        // Make this more permissive for debugging
        way.nodes.iter().any(|&node_id| {
            if let Some(node) = map_data.nodes.get(&node_id) {
                // Add some margin to the bounds to be more permissive
                let margin = 0.001; // ~100m at equator
                node.lon >= (bounds.min_lon - margin) && node.lon <= (bounds.max_lon + margin) &&
                node.lat >= (bounds.min_lat - margin) && node.lat <= (bounds.max_lat + margin)
            } else {
                false
            }
        })
    }
    
    fn point_in_bounds(&self, lon: f64, lat: f64, bounds: &VisibleBounds) -> bool {
        lon >= bounds.min_lon && lon <= bounds.max_lon &&
        lat >= bounds.min_lat && lat <= bounds.max_lat
    }
    
    fn get_way_style(&self, way: &crate::core::Way, style_manager: &StyleManager) -> ((u8, u8, u8), f32) {
        // Try to get style from style manager first
        if let Some(stylesheet) = style_manager.get_active_stylesheet() {
            for rule in &stylesheet.rules {
                if self.matches_way_selectors(way, &rule.selectors) {
                    let mut color = (128, 128, 128); // default gray
                    let mut width = 2.0;
                    
                    // Apply line color from rule
                    if let Some(line_color) = &rule.style.line_color {
                        color = (line_color.r, line_color.g, line_color.b);
                    }
                    
                    // Apply line width from rule
                    width = rule.style.line_width;
                    
                    return (color, width);
                }
            }
        }
        
        // Fallback to hardcoded styles if no stylesheet rules match
        if let Some(highway) = way.tags.get("highway") {
            match highway.as_str() {
                "motorway" => ((231, 114, 0), 6.0), // Use our stylesheet colors as fallback too
                "trunk" => ((255, 156, 0), 5.0),
                "primary" => ((255, 205, 0), 4.0),
                "secondary" => ((255, 230, 100), 3.5),
                "tertiary" => ((255, 245, 150), 3.0),
                "residential" => ((255, 255, 255), 2.5),
                "service" => ((240, 240, 240), 1.5),
                "footway" => ((200, 200, 200), 1.0),
                _ => ((200, 200, 200), 1.5),
            }
        } else if way.tags.contains_key("building") {
            ((218, 218, 218), 0.5) // Use stylesheet colors
        } else if way.tags.contains_key("waterway") {
            ((170, 211, 223), 2.0)
        } else if way.tags.get("natural") == Some(&"water".to_string()) {
            ((170, 211, 223), 1.0)
        } else {
            // Make default ways more visible
            ((100, 100, 100), 2.0)
        }
    }
    
    fn get_node_style(&self, node: &crate::core::Node, style_manager: &StyleManager) -> ((u8, u8, u8), f32) {
        // Try to get style from style manager first
        if let Some(stylesheet) = style_manager.get_active_stylesheet() {
            for rule in &stylesheet.rules {
                if self.matches_node_selectors(node, &rule.selectors) {
                    let mut color = (200, 100, 100); // default red
                    let mut size = 3.0;
                    
                    // Apply line color from rule (for point styling)
                    if let Some(line_color) = &rule.style.line_color {
                        color = (line_color.r, line_color.g, line_color.b);
                    }
                    
                    // Use line_width as point size for consistency
                    if rule.style.line_width > 0.0 {
                        size = rule.style.line_width;
                    }
                    
                    return (color, size);
                }
            }
        }
        
        // Fallback to hardcoded styles if no stylesheet rules match
        if node.tags.contains_key("amenity") {
            ((220, 20, 60), 4.0) // Red from stylesheet
        } else if node.tags.contains_key("shop") {
            ((30, 144, 255), 3.5) // Blue from stylesheet
        } else if node.tags.contains_key("tourism") {
            ((50, 205, 50), 3.5) // Green from stylesheet
        } else {
            // Make all nodes much smaller and less prominent - they should not dominate the map
            ((128, 128, 128), 1.0) // Gray and small
        }
    }
    
    pub fn zoom_to_fit(&mut self, map_data: &Option<MapData>) {
        if let Some(data) = map_data {
            if let Some(bounds) = self.calculate_data_bounds(data) {
                // Calculate center
                self.viewport.center_x = (bounds.min_lon + bounds.max_lon) / 2.0;
                self.viewport.center_y = (bounds.min_lat + bounds.max_lat) / 2.0;
                
                // Calculate scale to fit data with some padding
                let data_width = bounds.max_lon - bounds.min_lon;
                let data_height = bounds.max_lat - bounds.min_lat;
                
                if data_width > 0.0 && data_height > 0.0 {
                    // For geographic coordinates, we need reasonable scaling
                    // Geographic degrees are small numbers, so we need to scale appropriately
                    let scale_x = (self.viewport.width as f64 * 0.8) / data_width;
                    let scale_y = (self.viewport.height as f64 * 0.8) / data_height;
                    self.viewport.scale = scale_x.min(scale_y);
                    
                    // Clamp the scale to allow for very detailed viewing
                    // For local maps (few km), we want scales from hundreds to hundreds of thousands for building details
                    self.viewport.scale = self.viewport.scale.clamp(50.0, 500000.0);
                } else {
                    // For single points or very small areas, use a moderate zoom
                    self.viewport.scale = 1000.0;
                }
            }
        } else {
            // No data, reset to default
            self.viewport.scale = 10000.0;
            self.viewport.center_x = 0.0;
            self.viewport.center_y = 0.0;
        }
    }
    
    fn calculate_data_bounds(&self, map_data: &MapData) -> Option<VisibleBounds> {
        if map_data.nodes.is_empty() {
            return None;
        }
        
        // First pass: find the rough center to identify outliers
        let mut lats: Vec<f64> = Vec::new();
        let mut lons: Vec<f64> = Vec::new();
        
        for node in map_data.nodes.values() {
            // Skip nodes with obviously invalid coordinates
            if node.lat < -90.0 || node.lat > 90.0 || node.lon < -180.0 || node.lon > 180.0 {
                continue;
            }
            lats.push(node.lat);
            lons.push(node.lon);
        }
        
        if lats.is_empty() {
            return None;
        }
        
        // Sort to find median (more robust than mean for outliers)
        lats.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lons.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median_lat = lats[lats.len() / 2];
        let median_lon = lons[lons.len() / 2];
        
        // Second pass: exclude extreme outliers based on distance from median
        let mut min_lon = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lat = f64::NEG_INFINITY;
        
        let mut outlier_count = 0;
        let mut total_nodes = 0;
        let max_distance_from_median = 1.0; // 1 degree maximum distance from median
        
        for node in map_data.nodes.values() {
            total_nodes += 1;
            
            // Skip nodes with obviously invalid coordinates
            if node.lat < -90.0 || node.lat > 90.0 || node.lon < -180.0 || node.lon > 180.0 {
                outlier_count += 1;
                continue;
            }
            
            // Calculate distance from median
            let lat_distance = (node.lat - median_lat).abs();
            let lon_distance = (node.lon - median_lon).abs();
            
            // Exclude extreme outliers that are too far from the median
            if lat_distance > max_distance_from_median || lon_distance > max_distance_from_median {
                outlier_count += 1;
                continue;
            }
            
            min_lon = min_lon.min(node.lon);
            max_lon = max_lon.max(node.lon);
            min_lat = min_lat.min(node.lat);
            max_lat = max_lat.max(node.lat);
        }
        
        if outlier_count > 0 {
            println!("Excluded {} outlier coordinates out of {} total nodes", outlier_count, total_nodes);
        }
        
        // Check if we have valid bounds
        if min_lon == f64::INFINITY {
            return None;
        }
        
        // Add small padding if bounds are too small
        if (max_lon - min_lon) < 0.001 {
            min_lon -= 0.0005;
            max_lon += 0.0005;
        }
        if (max_lat - min_lat) < 0.001 {
            min_lat -= 0.0005;
            max_lat += 0.0005;
        }
        
        Some(VisibleBounds {
            min_lon,
            max_lon,
            min_lat,
            max_lat,
        })
    }
    
    fn matches_way_selectors(&self, way: &crate::core::Way, selectors: &[crate::parsers::stylesheet::FeatureSelector]) -> bool {
        for selector in selectors {
            match selector {
                crate::parsers::stylesheet::FeatureSelector::Tag { key, value } => {
                    if let Some(tag_value) = way.tags.get(key) {
                        if let Some(expected_value) = value {
                            if tag_value == expected_value {
                                return true;
                            }
                        } else {
                            // Key exists, no specific value required
                            return true;
                        }
                    }
                }
                crate::parsers::stylesheet::FeatureSelector::ElementType(element_type) => {
                    match element_type {
                        crate::parsers::stylesheet::ElementType::Way => return true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        false
    }
    
    fn matches_node_selectors(&self, node: &crate::core::Node, selectors: &[crate::parsers::stylesheet::FeatureSelector]) -> bool {
        for selector in selectors {
            match selector {
                crate::parsers::stylesheet::FeatureSelector::Tag { key, value } => {
                    if let Some(tag_value) = node.tags.get(key) {
                        if let Some(expected_value) = value {
                            if tag_value == expected_value {
                                return true;
                            }
                        } else {
                            // Key exists, no specific value required
                            return true;
                        }
                    }
                }
                crate::parsers::stylesheet::FeatureSelector::ElementType(element_type) => {
                    match element_type {
                        crate::parsers::stylesheet::ElementType::Node => return true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
struct VisibleBounds {
    min_lon: f64,
    max_lon: f64,
    min_lat: f64,
    max_lat: f64,
}

impl Default for MapView {
    fn default() -> Self {
        Self::new()
    }
}

impl MapView {
    fn handle_selection_input(&mut self, response: &Response, rect: Rect) {
        if response.drag_started() {
            // Start new selection on drag start
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                // Ensure mouse is within the map area
                if rect.contains(mouse_pos) {
                    self.selection_rect = Some(SelectionRect {
                        start_pos: mouse_pos,
                        current_pos: mouse_pos,
                        has_been_dragged: true, // Mark as dragged immediately
                    });
                }
            }
        } else if response.dragged() {
            // Update selection rectangle during drag
            if let (Some(ref mut selection), Some(mouse_pos)) = (&mut self.selection_rect, response.interact_pointer_pos()) {
                // Clamp mouse position to map bounds
                let clamped_pos = Pos2::new(
                    mouse_pos.x.clamp(rect.min.x, rect.max.x),
                    mouse_pos.y.clamp(rect.min.y, rect.max.y)
                );
                selection.current_pos = clamped_pos;
            }
        } else if response.drag_stopped() {
            // Complete selection and zoom to area
            if let Some(selection) = self.selection_rect.take() {
                self.zoom_to_selection(&selection, rect);
                self.selection_mode = false; // Exit selection mode after zoom
                println!("Rectangle zoom selection completed");
            }
        }
        
        // Cancel selection on right click
        if response.secondary_clicked() {
            self.selection_rect = None;
            self.selection_mode = false;
            println!("Rectangle zoom selection cancelled");
        }
    }
    
    fn zoom_to_selection(&mut self, selection: &SelectionRect, rect: Rect) {
        let min_x = selection.start_pos.x.min(selection.current_pos.x);
        let max_x = selection.start_pos.x.max(selection.current_pos.x);
        let min_y = selection.start_pos.y.min(selection.current_pos.y);
        let max_y = selection.start_pos.y.max(selection.current_pos.y);
        
        // Require minimum rectangle size to avoid accidental tiny selections
        let selection_width = max_x - min_x;
        let selection_height = max_y - min_y;
        
        if selection_width < 10.0 || selection_height < 10.0 {
            return;
        }
        
        // Convert screen coordinates to map coordinates
        let center_x = rect.center().x;
        let center_y = rect.center().y;
        
        // Calculate map coordinates for selection bounds
        let min_map_x = self.viewport.center_x + ((min_x - center_x) as f64) / self.viewport.scale;
        let max_map_x = self.viewport.center_x + ((max_x - center_x) as f64) / self.viewport.scale;
        let min_map_y = self.viewport.center_y - ((max_y - center_y) as f64) / self.viewport.scale; // Flip Y
        let max_map_y = self.viewport.center_y - ((min_y - center_y) as f64) / self.viewport.scale; // Flip Y
        
        // Calculate new center
        self.viewport.center_x = (min_map_x + max_map_x) / 2.0;
        self.viewport.center_y = (min_map_y + max_map_y) / 2.0;
        
        // Calculate new scale to fit the selection
        let map_width = max_map_x - min_map_x;
        let map_height = max_map_y - min_map_y;
        
        if map_width > 0.0 && map_height > 0.0 {
            let scale_x = (rect.width() as f64 * 0.9) / map_width; // 90% to leave some padding
            let scale_y = (rect.height() as f64 * 0.9) / map_height;
            self.viewport.scale = scale_x.min(scale_y);
            
            // Clamp to very detailed zoom levels
            self.viewport.scale = self.viewport.scale.clamp(0.001, 500000.0);
        }
    }
    
    fn draw_selection_rectangle(&self, ui: &mut Ui, rect: Rect) {
        if let Some(selection) = &self.selection_rect {
            // Only draw the rectangle if it has been dragged (not just a single click)
            if selection.has_been_dragged {
                let painter = ui.painter_at(rect);
                
                let min_x = selection.start_pos.x.min(selection.current_pos.x);
                let max_x = selection.start_pos.x.max(selection.current_pos.x);
                let min_y = selection.start_pos.y.min(selection.current_pos.y);
                let max_y = selection.start_pos.y.max(selection.current_pos.y);
                
                let selection_rect = Rect::from_min_max(
                    Pos2::new(min_x, min_y),
                    Pos2::new(max_x, max_y)
                );
                
                // Draw selection rectangle with semi-transparent fill and border
                painter.rect(
                    selection_rect,
                    2.0,
                    Color32::from_rgba_unmultiplied(0, 150, 255, 50), // Light blue with transparency
                    egui::Stroke::new(2.0, Color32::from_rgb(0, 120, 255)) // Blue border
                );
                
                // Draw corner markers for better visibility
                let corner_size = 4.0;
                let corner_color = Color32::from_rgb(0, 120, 255);
                
                // Top-left corner
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(min_x, min_y), Vec2::splat(corner_size)),
                    0.0,
                    corner_color
                );
                
                // Top-right corner
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(max_x, min_y), Vec2::splat(corner_size)),
                    0.0,
                    corner_color
                );
                
                // Bottom-left corner
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(min_x, max_y), Vec2::splat(corner_size)),
                    0.0,
                    corner_color
                );
                
                // Bottom-right corner
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(max_x, max_y), Vec2::splat(corner_size)),
                    0.0,
                    corner_color
                );
            }
        }
        
        // Show instruction text when in selection mode
        if self.selection_mode {
            let painter = ui.painter_at(rect);
            let instruction_text = if self.selection_rect.is_some() {
                "Release to zoom to selected area"
            } else {
                "Click and drag to select zoom area"
            };
            
            painter.text(
                rect.min + Vec2::new(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                instruction_text,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 120, 255)
            );
        }
    }
    
    // Google Maps-style specialized drawing methods
    
    fn draw_water_areas(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            // Check if it's a water feature
            let is_water = way.tags.get("natural") == Some(&"water".to_string()) ||
                          way.tags.get("waterway").is_some() ||
                          way.tags.get("water").is_some();
            
            if is_water && way.is_closed {
                let points: Vec<Pos2> = way.nodes
                    .iter()
                    .filter_map(|&node_id| map_data.nodes.get(&node_id))
                    .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                    .collect();
                
                if points.len() > 2 {
                    // Google Maps water color: light blue
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        Color32::from_rgb(170, 218, 255), // Light blue fill
                        egui::Stroke::new(1.0, Color32::from_rgb(110, 180, 240)), // Slightly darker border
                    ));
                }
            }
        }
    }
    
    fn draw_landuse_areas(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) || !way.is_closed {
                continue;
            }
            
            // Check if this way has landuse, leisure, or natural tags we care about
            let mut should_draw = false;
            let mut fill_color = Color32::TRANSPARENT;
            let mut stroke_color = Color32::TRANSPARENT;
            
            if let Some(landuse) = way.tags.get("landuse") {
                match landuse.as_str() {
                    "forest" | "wood" => {
                        fill_color = Color32::from_rgb(200, 220, 188);
                        stroke_color = Color32::from_rgb(180, 200, 168);
                        should_draw = true;
                    },
                    "grass" | "meadow" => {
                        fill_color = Color32::from_rgb(220, 240, 200);
                        stroke_color = Color32::from_rgb(200, 220, 180);
                        should_draw = true;
                    },
                    "residential" => {
                        fill_color = Color32::from_rgb(240, 240, 235);
                        stroke_color = Color32::from_rgb(220, 220, 215);
                        should_draw = true;
                    },
                    "commercial" => {
                        fill_color = Color32::from_rgb(245, 240, 235);
                        stroke_color = Color32::from_rgb(225, 220, 215);
                        should_draw = true;
                    },
                    "industrial" => {
                        fill_color = Color32::from_rgb(235, 235, 240);
                        stroke_color = Color32::from_rgb(215, 215, 220);
                        should_draw = true;
                    },
                    _ => {}
                }
            } else if let Some(leisure) = way.tags.get("leisure") {
                match leisure.as_str() {
                    "park" | "garden" => {
                        fill_color = Color32::from_rgb(200, 240, 200);
                        stroke_color = Color32::from_rgb(180, 220, 180);
                        should_draw = true;
                    },
                    "playground" => {
                        fill_color = Color32::from_rgb(255, 245, 220);
                        stroke_color = Color32::from_rgb(235, 225, 200);
                        should_draw = true;
                    },
                    _ => {}
                }
            } else if let Some(natural) = way.tags.get("natural") {
                match natural.as_str() {
                    "wood" | "forest" => {
                        fill_color = Color32::from_rgb(200, 220, 188);
                        stroke_color = Color32::from_rgb(180, 200, 168);
                        should_draw = true;
                    },
                    "grassland" => {
                        fill_color = Color32::from_rgb(220, 240, 200);
                        stroke_color = Color32::from_rgb(200, 220, 180);
                        should_draw = true;
                    },
                    _ => {}
                }
            }
            
            if !should_draw {
                continue;
            }
            
            let points: Vec<Pos2> = way.nodes
                .iter()
                .filter_map(|&node_id| map_data.nodes.get(&node_id))
                .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                .collect();
            
            if points.len() > 2 {
                painter.add(egui::Shape::convex_polygon(
                    points,
                    fill_color,
                    egui::Stroke::new(0.5, stroke_color),
                ));
            }
        }
    }
    
    fn draw_buildings(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            if way.tags.contains_key("building") && way.is_closed {
                let points: Vec<Pos2> = way.nodes
                    .iter()
                    .filter_map(|&node_id| map_data.nodes.get(&node_id))
                    .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                    .collect();
                
                if points.len() > 2 {
                    // Google Maps building style: light gray with subtle shadow effect
                    let building_color = Color32::from_rgb(228, 228, 228);
                    let building_stroke = Color32::from_rgb(200, 200, 200);
                    
                    // Draw shadow first (slightly offset)
                    if self.viewport.scale > 5000.0 { // Only at high zoom levels
                        let shadow_points: Vec<Pos2> = points.iter()
                            .map(|p| Pos2::new(p.x + 1.0, p.y + 1.0))
                            .collect();
                        painter.add(egui::Shape::convex_polygon(
                            shadow_points,
                            Color32::from_rgba_unmultiplied(0, 0, 0, 20), // Subtle shadow
                            egui::Stroke::NONE,
                        ));
                    }
                    
                    // Draw building
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        building_color,
                        egui::Stroke::new(0.5, building_stroke),
                    ));
                }
            }
        }
    }
    
    fn draw_road_casings(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            if let Some(highway) = way.tags.get("highway") {
                let (casing_width, casing_color) = self.get_road_casing_style(highway);
                
                if casing_width > 0.0 {
                    let points: Vec<Pos2> = way.nodes
                        .iter()
                        .filter_map(|&node_id| map_data.nodes.get(&node_id))
                        .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                        .collect();
                    
                    if points.len() >= 2 {
                        painter.add(egui::Shape::line(
                            points,
                            egui::Stroke::new(casing_width, casing_color),
                        ));
                    }
                }
            }
        }
    }
    
    fn draw_road_fills(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            if let Some(highway) = way.tags.get("highway") {
                let (width, color) = self.get_road_fill_style(highway);
                
                let points: Vec<Pos2> = way.nodes
                    .iter()
                    .filter_map(|&node_id| map_data.nodes.get(&node_id))
                    .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                    .collect();
                
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new(width, color),
                    ));
                }
            }
        }
    }
    
    fn draw_railways(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            if way.tags.get("railway").is_some() {
                let points: Vec<Pos2> = way.nodes
                    .iter()
                    .filter_map(|&node_id| map_data.nodes.get(&node_id))
                    .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                    .collect();
                
                if points.len() >= 2 {
                    // Draw railway as dashed line
                    painter.add(egui::Shape::dashed_line(
                        &points,
                        egui::Stroke::new(2.0, Color32::from_rgb(120, 120, 120)),
                        10.0,
                        5.0,
                    ));
                }
            }
        }
    }
    
    fn draw_points_of_interest(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        // Only draw POIs at reasonable zoom levels
        if self.viewport.scale < 1000.0 {
            return;
        }
        
        for node in map_data.nodes.values() {
            if !self.point_in_bounds(node.lon, node.lat, visible_bounds) {
                continue;
            }
            
            let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
            let (color, size, icon) = self.get_poi_style(node);
            
            if size > 0.0 {
                // Draw POI marker with Google Maps-style appearance
                painter.circle_filled(
                    screen_pos,
                    size + 1.0, // Slightly larger background
                    Color32::WHITE, // White background
                );
                painter.circle_filled(
                    screen_pos,
                    size,
                    color,
                );
                
                // Draw icon or letter if available
                if let Some(icon_text) = icon {
                    painter.text(
                        screen_pos,
                        egui::Align2::CENTER_CENTER,
                        icon_text,
                        egui::FontId::proportional(8.0),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
    
    fn draw_text_labels(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        // Only show labels at higher zoom levels
        if self.viewport.scale < 2000.0 {
            return;
        }
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            if let Some(name) = way.tags.get("name") {
                // Calculate center point for label
                let center_node_id = way.nodes.get(way.nodes.len() / 2).copied();
                if let Some(node_id) = center_node_id {
                    if let Some(node) = map_data.nodes.get(&node_id) {
                        let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
                        
                        let (font_size, font_color) = self.get_label_style(way);
                        
                        // Draw text with subtle background for readability
                        painter.text(
                            screen_pos + Vec2::new(0.0, -2.0),
                            egui::Align2::CENTER_CENTER,
                            name,
                            egui::FontId::proportional(font_size),
                            font_color,
                        );
                    }
                }
            }
        }
        
        // Draw node labels (POI names)
        if self.viewport.scale > 5000.0 {
            for node in map_data.nodes.values() {
                if !self.point_in_bounds(node.lon, node.lat, visible_bounds) {
                    continue;
                }
                
                if let Some(name) = node.tags.get("name") {
                    if node.tags.contains_key("amenity") || node.tags.contains_key("shop") || node.tags.contains_key("tourism") {
                        let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
                        
                        painter.text(
                            screen_pos + Vec2::new(8.0, 0.0), // Offset to the right of POI marker
                            egui::Align2::LEFT_CENTER,
                            name,
                            egui::FontId::proportional(9.0),
                            Color32::from_rgb(60, 60, 60),
                        );
                    }
                }
            }
        }
    }
    
    // Google Maps-style helper methods for styling
    
    fn get_road_casing_style(&self, highway: &str) -> (f32, Color32) {
        match highway {
            "motorway" => (8.0, Color32::from_rgb(140, 100, 0)),
            "trunk" => (7.0, Color32::from_rgb(160, 120, 0)),
            "primary" => (6.0, Color32::from_rgb(180, 140, 0)),
            "secondary" => (5.0, Color32::from_rgb(180, 160, 80)),
            "tertiary" => (4.0, Color32::from_rgb(180, 180, 120)),
            "residential" => (3.5, Color32::from_rgb(200, 200, 200)),
            "service" => (2.5, Color32::from_rgb(210, 210, 210)),
            _ => (0.0, Color32::TRANSPARENT), // No casing for minor roads
        }
    }
    
    fn get_road_fill_style(&self, highway: &str) -> (f32, Color32) {
        match highway {
            "motorway" => (6.0, Color32::from_rgb(231, 170, 56)), // Orange
            "trunk" => (5.5, Color32::from_rgb(255, 196, 56)), // Light orange
            "primary" => (5.0, Color32::from_rgb(255, 220, 56)), // Yellow
            "secondary" => (4.0, Color32::from_rgb(255, 240, 120)), // Light yellow
            "tertiary" => (3.5, Color32::WHITE), // White
            "residential" => (3.0, Color32::WHITE), // White
            "service" => (2.0, Color32::from_rgb(245, 245, 245)), // Very light gray
            "footway" | "path" => (1.5, Color32::from_rgb(220, 180, 140)), // Brown
            "cycleway" => (1.5, Color32::from_rgb(100, 150, 255)), // Blue
            _ => (2.0, Color32::from_rgb(230, 230, 230)), // Light gray default
        }
    }
    
    fn get_poi_style(&self, node: &crate::core::Node) -> (Color32, f32, Option<String>) {
        if let Some(amenity) = node.tags.get("amenity") {
            match amenity.as_str() {
                "restaurant" | "cafe" | "fast_food" => (Color32::from_rgb(220, 80, 40), 4.0, Some("🍽".to_string())),
                "hospital" => (Color32::from_rgb(220, 20, 60), 5.0, Some("H".to_string())),
                "school" => (Color32::from_rgb(100, 150, 255), 4.0, Some("🎓".to_string())),
                "bank" => (Color32::from_rgb(50, 150, 50), 4.0, Some("$".to_string())),
                "fuel" => (Color32::from_rgb(255, 150, 0), 4.0, Some("⛽".to_string())),
                _ => (Color32::from_rgb(150, 150, 150), 3.0, None),
            }
        } else if let Some(shop) = node.tags.get("shop") {
            (Color32::from_rgb(100, 150, 255), 3.5, Some("🛍".to_string()))
        } else if let Some(tourism) = node.tags.get("tourism") {
            match tourism.as_str() {
                "hotel" => (Color32::from_rgb(150, 100, 200), 4.0, Some("🏨".to_string())),
                "museum" => (Color32::from_rgb(200, 150, 100), 4.0, Some("🏛".to_string())),
                "attraction" => (Color32::from_rgb(255, 100, 150), 4.0, Some("★".to_string())),
                _ => (Color32::from_rgb(200, 100, 150), 3.5, None),
            }
        } else {
            (Color32::TRANSPARENT, 0.0, None) // Don't draw regular nodes
        }
    }
    
    fn get_label_style(&self, way: &crate::core::Way) -> (f32, Color32) {
        if let Some(highway) = way.tags.get("highway") {
            match highway.as_str() {
                "motorway" | "trunk" => (11.0, Color32::WHITE),
                "primary" => (10.0, Color32::BLACK),
                "secondary" | "tertiary" => (9.0, Color32::from_rgb(60, 60, 60)),
                _ => (8.0, Color32::from_rgb(80, 80, 80)),
            }
        } else if way.tags.contains_key("building") {
            (8.0, Color32::from_rgb(100, 100, 100))
        } else {
            (8.0, Color32::from_rgb(80, 80, 80))
        }
    }
    
    fn draw_viewport_info(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        
        // Draw scale info in bottom-right corner (Google Maps style)
        let scale_text = format!("{:.0}m/px", 1.0 / self.viewport.scale * 111320.0); // Approximate meters per pixel
        
        let text_pos = rect.max - Vec2::new(15.0, 15.0);
        
        // Draw semi-transparent background
        let text_size = painter.layout_no_wrap(
            scale_text.clone(),
            egui::FontId::monospace(9.0),
            Color32::WHITE,
        ).rect.size();
        
        let bg_rect = Rect::from_min_size(
            text_pos - Vec2::new(text_size.x + 6.0, text_size.y + 4.0),
            text_size + Vec2::new(8.0, 6.0),
        );
        
        painter.rect_filled(
            bg_rect,
            3.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 160),
        );
        
        painter.text(
            text_pos - Vec2::new(text_size.x + 2.0, text_size.y + 1.0),
            egui::Align2::LEFT_TOP,
            scale_text,
            egui::FontId::monospace(9.0),
            Color32::WHITE,
        );
    }
}
