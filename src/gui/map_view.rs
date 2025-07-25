use crate::core::MapData;
use crate::gui::{Tool, GuiState};
use crate::rendering::MapRenderer;
use crate::styles::loader::StyleManager;
use egui::{Ui, Response, Sense, Vec2, Pos2, Rect, Color32};
use log::{debug, info, warn};
use std::collections::HashMap;

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
    /// Selected map element (for style editing)
    selected_element: Option<SelectedElement>,
}

#[derive(Debug, Clone)]
pub struct SelectedElement {
    pub element_type: ElementType,
    pub element_id: i64,
    pub tags: HashMap<String, String>,
    pub style_info: StyleInfo,
}

#[derive(Debug, Clone)]
pub enum ElementType {
    Way,
    Node,
    Relation,
}

#[derive(Debug, Clone)]
pub struct StyleInfo {
    pub category: String,  // e.g., "highway", "building", "natural", etc.
    pub subcategory: String,  // e.g., "primary", "residential", "park", etc.
    pub toml_section: String,  // Section name in TOML file
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
            selected_element: None,
        }
    }
    
    /// Get the currently selected element
    pub fn get_selected_element(&self) -> Option<&SelectedElement> {
        self.selected_element.as_ref()
    }
    
    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selected_element = None;
    }
    
    pub fn show(&mut self, ui: &mut Ui, map_data: &Option<MapData>, renderer: &MapRenderer, style_manager: &StyleManager, gui_state: &GuiState, modal_is_open: bool) -> (Response, Option<Pos2>) {
        let available_size = ui.available_size();
        let (rect, mut response) = ui.allocate_exact_size(available_size, Sense::click_and_drag().union(Sense::hover()));
        
        // Set cursor icon based on current tool
        match gui_state.current_tool {
            Tool::Pan => {
                response = response.on_hover_cursor(egui::CursorIcon::Grab);
            },
            Tool::RectangleZoom if self.selection_mode => {
                response = response.on_hover_cursor(egui::CursorIcon::Crosshair);
            },
            _ => {
                // Default cursor for other tools
            }
        }
        
        // Update viewport size
        self.viewport.width = rect.width();
        self.viewport.height = rect.height();
        
        // Handle input (pass the rect for coordinate conversion)
        self.handle_input(ui, &response, rect, map_data, gui_state, modal_is_open);
        
        // Draw the map
        self.draw_map(ui, rect, map_data, renderer, style_manager, gui_state);
        
        // Get hover position before moving response
        let hover_pos = response.hover_pos();
        
        // Return response and hover position
        (response, hover_pos)
    }
    
    /// Toggle rectangle selection mode on/off
    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = !self.selection_mode;
        self.selection_rect = None; // Clear any existing selection
        debug!("Rectangle zoom selection mode: {}", if self.selection_mode { "ON" } else { "OFF" });
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
    
    /// Center the map on specific coordinates
    pub fn center_on_coordinates(&mut self, lat: f64, lon: f64) {
        self.viewport.center_x = lon;
        self.viewport.center_y = lat;
    }
    
    /// Center the map on specific coordinates with a specific zoom level
    pub fn center_on_coordinates_with_zoom(&mut self, lat: f64, lon: f64, zoom_scale: f64) {
        self.viewport.center_x = lon;
        self.viewport.center_y = lat;
        self.viewport.scale = zoom_scale.clamp(0.001, 500000.0);
    }
    
    /// Get viewport information for export (center coordinates and scale)
    pub fn get_viewport_info(&self) -> (f64, f64, f64) {
        (self.viewport.center_x, self.viewport.center_y, self.viewport.scale)
    }
    
    /// Get detailed status information for the status bar
    pub fn get_status_info(&self, hover_pos: Option<Pos2>, rect: Rect, map_data: &Option<crate::core::MapData>) -> String {
        let mut status_parts = Vec::new();
        
        // Add zoom level and scale info
        let scale_meters_per_pixel = 1.0 / self.viewport.scale * 111320.0; // Approximate meters per pixel
        status_parts.push(format!("Zoom: {:.1}x", self.viewport.scale / 1000.0));
        status_parts.push(format!("Scale: {:.0}m/px", scale_meters_per_pixel));
        
        // Add center coordinates
        status_parts.push(format!("Center: {:.6}, {:.6}", self.viewport.center_x, self.viewport.center_y));
        
        // Add mouse coordinates if available
        if let Some(mouse_pos) = hover_pos {
            let (mouse_lon, mouse_lat) = self.screen_to_map(mouse_pos, rect);
            status_parts.push(format!("Mouse: {:.6}, {:.6}", mouse_lon, mouse_lat));
        }
        
        // Add map data statistics
        if let Some(data) = map_data {
            status_parts.push(format!("Features: {}/{}", data.ways.len(), data.nodes.len()));
        }
        
        status_parts.join(" | ")
    }

    fn handle_input(&mut self, ui: &mut Ui, response: &Response, rect: Rect, map_data: &Option<MapData>, gui_state: &GuiState, modal_is_open: bool) {
        // Skip scroll handling if a modal is open to prevent interference
        if !modal_is_open {
            // Handle mouse wheel for zooming FIRST (should work in all modes)
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
            if scroll_delta.y != 0.0 {
                debug!("Zoom event detected: scroll_delta.y = {}", scroll_delta.y);
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
                    debug!("Zoom applied: {} -> {}", old_scale, self.viewport.scale);
                    
                    // Adjust center to zoom towards mouse position
                    self.viewport.center_x = map_x - rel_x / self.viewport.scale;
                    self.viewport.center_y = map_y - rel_y / self.viewport.scale;
                } else {
                    // Simple zoom at center
                    let old_scale = self.viewport.scale;
                    self.viewport.scale *= zoom_factor;
                    debug!("Simple zoom applied: {} -> {}", old_scale, self.viewport.scale);
                }
                
                // Clamp zoom level to allow for very detailed viewing
                self.viewport.scale = self.viewport.scale.clamp(0.001, 500000.0);
            }
        }

        // Handle rectangle selection mode
        if self.selection_mode {
            self.handle_selection_input(response, rect);
            return; // Skip normal panning/zooming in selection mode
        }
        
        // Handle element selection in Select mode
        if gui_state.current_tool == Tool::Select && response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                self.handle_element_selection(click_pos, rect, map_data);
            }
            return; // Skip panning in select mode
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
    
    fn draw_map(&self, ui: &mut Ui, rect: Rect, map_data: &Option<MapData>, renderer: &MapRenderer, style_manager: &StyleManager, gui_state: &GuiState) {
        let painter = ui.painter_at(rect);
        
        // Draw background using the style from TOML config
        let style = style_manager.get_current_style();
        let bg_color = Self::hex_to_rgb(&style.background.color);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(bg_color.0, bg_color.1, bg_color.2));
        
        if let Some(data) = map_data {
            // Calculate visible bounds
            let visible_bounds = self.calculate_visible_bounds(rect);
            
            // Draw map features in proper order (like Google Maps) based on layer visibility
            // 1. Water bodies and areas (lowest layer)
            if gui_state.show_water {
                self.draw_water_areas(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 2. Land use areas (parks, forests, etc.)
            if gui_state.show_landuse {
                self.draw_landuse_areas(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 3. Buildings (with shadows for 3D effect)
            if gui_state.show_buildings {
                self.draw_buildings(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 4. Road casings (dark outlines first)
            if gui_state.show_roads {
                self.draw_road_casings(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 5. Road fills (lighter colors on top)
            if gui_state.show_roads {
                self.draw_road_fills(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 6. Railways and other transport
            if gui_state.show_roads {
                self.draw_railways(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 7. Points of Interest (POIs)
            if gui_state.show_pois {
                self.draw_pois(ui, rect, data, &visible_bounds, style_manager);
            }
            
            // 8. Text labels (highest layer)
            self.draw_text_labels(ui, rect, data, &visible_bounds, style_manager);
            
            // 9. Selection highlight (topmost layer)
            self.draw_selection_highlight(ui, rect, data);
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
    }
    
    fn map_to_screen(&self, lon: f64, lat: f64, rect: Rect) -> Pos2 {
        let x = (lon - self.viewport.center_x) * self.viewport.scale + (rect.width() / 2.0) as f64;
        let y = -(lat - self.viewport.center_y) * self.viewport.scale + (rect.height() / 2.0) as f64;
        
        Pos2::new(
            rect.min.x + x as f32,
            rect.min.y + y as f32,
        )
    }
    
    fn screen_to_map(&self, screen_pos: Pos2, rect: Rect) -> (f64, f64) {
        let rel_x = (screen_pos.x - rect.min.x) as f64 - (rect.width() / 2.0) as f64;
        let rel_y = (screen_pos.y - rect.min.y) as f64 - (rect.height() / 2.0) as f64;
        
        let lon = self.viewport.center_x + rel_x / self.viewport.scale;
        let lat = self.viewport.center_y - rel_y / self.viewport.scale; // Flip Y
        
        (lon, lat)
    }
    
    fn calculate_visible_bounds(&self, rect: Rect) -> VisibleBounds {
        let half_width = (rect.width() / 2.0) as f64 / self.viewport.scale;
        let half_height = (rect.height() / 2.0) as f64 / self.viewport.scale;
        
        let bounds = VisibleBounds {
            min_lon: self.viewport.center_x - half_width,
            max_lon: self.viewport.center_x + half_width,
            min_lat: self.viewport.center_y - half_height,
            max_lat: self.viewport.center_y + half_height,
        };
        
        // Debug: Print viewport and bounds information
        debug!("Viewport - center: ({:.6}, {:.6}), scale: {:.6}", 
                 self.viewport.center_x, self.viewport.center_y, self.viewport.scale);
        debug!("Calculated bounds - lat: {:.6} to {:.6}, lon: {:.6} to {:.6}",
                 bounds.min_lat, bounds.max_lat, bounds.min_lon, bounds.max_lon);
        
        bounds
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
        let style = style_manager.get_current_style();
        
        // Check for roads first
        if let Some(highway) = way.tags.get("highway") {
            let (color_str, width, _, _) = style.get_road_style(highway);
            return (Self::hex_to_rgb(color_str), width);
        }
        
        // Check for buildings
        if way.tags.contains_key("building") {
            return (Self::hex_to_rgb(&style.buildings.fill), style.buildings.stroke_width);
        }
        
        // Check for waterways
        if way.tags.contains_key("waterway") {
            return (Self::hex_to_rgb(&style.water.color), 2.0);
        }
        
        // Check for natural water features
        if way.tags.get("natural") == Some(&"water".to_string()) {
            return (Self::hex_to_rgb(&style.water.color), 1.0);
        }
        
        // Check for landuse
        if let Some(landuse) = way.tags.get("landuse") {
            if let Some(color) = style.get_landuse_color(landuse) {
                return (Self::hex_to_rgb(color), 1.0);
            }
        }
        
        // Check for leisure
        if let Some(leisure) = way.tags.get("leisure") {
            if let Some(color) = style.get_leisure_color(leisure) {
                return (Self::hex_to_rgb(color), 1.0);
            }
        }
        
        // Check for natural features
        if let Some(natural) = way.tags.get("natural") {
            if let Some(color) = style.get_natural_color(natural) {
                return (Self::hex_to_rgb(color), 1.0);
            }
        }
        
        // Default fallback
        ((100, 100, 100), 1.0)
    }
    
    fn get_node_style(&self, node: &crate::core::Node, style_manager: &StyleManager) -> ((u8, u8, u8), f32) {
        let style = style_manager.get_current_style();
        
        // Check for amenities
        if let Some(amenity) = node.tags.get("amenity") {
            let (color_str, radius) = style.get_poi_style(amenity);
            return (Self::hex_to_rgb(color_str), radius);
        }
        
        // Check for shops
        if let Some(shop) = node.tags.get("shop") {
            let (color_str, radius) = style.get_poi_style(shop);
            return (Self::hex_to_rgb(color_str), radius);
        }
        
        // Check for tourism
        if let Some(tourism) = node.tags.get("tourism") {
            let (color_str, radius) = style.get_poi_style(tourism);
            return (Self::hex_to_rgb(color_str), radius);
        }
        
        // Default small gray node
        ((128, 128, 128), 1.0)
    }
    
    /// Convert hex color string to RGB tuple
    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r, g, b);
            }
        }
        // Fallback to gray if parsing fails
        (128, 128, 128)
    }
    
    pub fn zoom_to_fit(&mut self, map_data: &Option<MapData>) {
        if let Some(data) = map_data {
            if let Some(data_bounds) = self.calculate_data_bounds(data) {
                // Debug multiple coordinates to check rendering
                let coords = vec![
                    (48.9443224247288, 2.177457844649215, "Boulevard de Bezons coord 1"),
                    (48.94396813214317, 2.1806281043179876, "Rue Georges Bernanos coord"),
                    (48.9448515, 2.1814650, "Georges Bernanos node 331847783"),
                    (48.9445555, 2.1812572, "Georges Bernanos node 2558905075"),
                ];
                
                // Use the first coordinate as center
                let (target_lat, target_lon, desc) = coords[0];
                
                debug!("zoom_to_fit - centering on: {} at lat={:.6}, lon={:.6}", desc, target_lat, target_lon);
                
                self.viewport.center_x = target_lon;
                self.viewport.center_y = target_lat;
                
                // Use moderate zoom to see wider area
                self.viewport.scale = 50000.0;
                
                debug!("zoom_to_fit - set viewport center to ({:.6}, {:.6}) with scale {:.1}", 
                         self.viewport.center_x, self.viewport.center_y, self.viewport.scale);
                
                // Check which roads exist near these coordinates
                for (lat, lon, desc) in coords {
                    debug!("Checking roads near {}: lat={:.6}, lon={:.6}", desc, lat, lon);
                    self.debug_roads_near_point(data, lat, lon, 0.002); // ~200m radius
                }
            }
        } else {
            // No data, reset to default
            self.viewport.scale = 12000.0;
            self.viewport.center_x = 0.0;
            self.viewport.center_y = 0.0;
        }
    }
    
    fn calculate_data_bounds(&self, map_data: &MapData) -> Option<DataBounds> {
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
        
        debug!("calculate_data_bounds: total nodes: {}, median lat: {:.6}, median lon: {:.6}", 
                 lats.len(), median_lat, median_lon);
        
        // Second pass: exclude extreme outliers based on distance from median
        let mut min_lon = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lat = f64::NEG_INFINITY;
        
        let mut outlier_count = 0;
        let mut total_nodes = 0;
        
        // Calculate the interquartile range for adaptive outlier detection
        let q1_lat = lats[lats.len() / 4];
        let q3_lat = lats[3 * lats.len() / 4];
        let q1_lon = lons[lons.len() / 4];
        let q3_lon = lons[3 * lons.len() / 4];
        
        let iqr_lat = q3_lat - q1_lat;
        let iqr_lon = q3_lon - q1_lon;
        
        // Use a more selective outlier threshold - 1.5 times the IQR beyond Q1/Q3 (standard statistical outlier detection)
        // But add a reasonable maximum threshold to handle datasets with extreme spread
        let lat_outlier_threshold = (1.5 * iqr_lat).min(1.0).max(0.01); // Max 1 degree, min 0.01 degree
        let lon_outlier_threshold = (1.5 * iqr_lon).min(1.5).max(0.01); // Max 1.5 degrees, min 0.01 degree
        
        debug!("calculate_data_bounds: IQR-based thresholds - lat: {:.6}, lon: {:.6}", 
                 lat_outlier_threshold, lon_outlier_threshold);
        debug!("calculate_data_bounds: Q1-Q3 ranges - lat: {:.6} to {:.6}, lon: {:.6} to {:.6}", 
                 q1_lat, q3_lat, q1_lon, q3_lon);
        
        for node in map_data.nodes.values() {
            total_nodes += 1;
            
            // Skip nodes with obviously invalid coordinates
            if node.lat < -90.0 || node.lat > 90.0 || node.lon < -180.0 || node.lon > 180.0 {
                outlier_count += 1;
                continue;
            }
            
            // Calculate distance from quartiles for more robust outlier detection
            let lat_distance_from_q1 = if node.lat < q1_lat { q1_lat - node.lat } else { 0.0 };
            let lat_distance_from_q3 = if node.lat > q3_lat { node.lat - q3_lat } else { 0.0 };
            let lon_distance_from_q1 = if node.lon < q1_lon { q1_lon - node.lon } else { 0.0 };
            let lon_distance_from_q3 = if node.lon > q3_lon { node.lon - q3_lon } else { 0.0 };
            
            // Exclude extreme outliers based on IQR method
            if lat_distance_from_q1 > lat_outlier_threshold || lat_distance_from_q3 > lat_outlier_threshold ||
               lon_distance_from_q1 > lon_outlier_threshold || lon_distance_from_q3 > lon_outlier_threshold {
                outlier_count += 1;
                continue;
            }
            
            min_lon = min_lon.min(node.lon);
            max_lon = max_lon.max(node.lon);
            min_lat = min_lat.min(node.lat);
            max_lat = max_lat.max(node.lat);
        }
        
        if outlier_count > 0 {
            debug!("Excluded {} outlier coordinates out of {} total nodes", outlier_count, total_nodes);
        }
        
        // Check if we have valid bounds
        if min_lon == f64::INFINITY {
            warn!("calculate_data_bounds: No valid bounds found after outlier filtering");
            return None;
        }
        
        debug!("calculate_data_bounds: raw bounds - lat: {:.6} to {:.6}, lon: {:.6} to {:.6}", 
                 min_lat, max_lat, min_lon, max_lon);
        
        // Add small padding if bounds are too small
        if (max_lon - min_lon) < 0.001 {
            min_lon -= 0.0005;
            max_lon += 0.0005;
        }
        if (max_lat - min_lat) < 0.001 {
            min_lat -= 0.0005;
            max_lat += 0.0005;
        }
        
        debug!("calculate_data_bounds: final bounds - lat: {:.6} to {:.6}, lon: {:.6} to {:.6}", 
                 min_lat, max_lat, min_lon, max_lon);
        
        Some(DataBounds {
            bounds: VisibleBounds {
                min_lon,
                max_lon,
                min_lat,
                max_lat,
            },
            median_lat,
            median_lon,
        })
    }
    
    fn debug_roads_near_point(&self, map_data: &MapData, target_lat: f64, target_lon: f64, radius: f64) {
        debug!("Searching for roads near lat={:.6}, lon={:.6}, radius={:.6}", target_lat, target_lon, radius);
        
        let mut found_roads = Vec::new();
        
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                // Check if any node in this way is within the radius
                let mut has_nearby_node = false;
                let mut min_distance = f64::INFINITY;
                
                for &node_id in &way.nodes {
                    if let Some(node) = map_data.nodes.get(&node_id) {
                        let lat_diff = node.lat - target_lat;
                        let lon_diff = node.lon - target_lon;
                        let distance = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt();
                        
                        if distance < min_distance {
                            min_distance = distance;
                        }
                        
                        if distance <= radius {
                            has_nearby_node = true;
                        }
                    }
                }
                
                if has_nearby_node {
                    let name = way.tags.get("name").cloned().unwrap_or_else(|| "<unnamed>".to_string());
                    found_roads.push((way.id, name, highway.clone(), min_distance));
                }
            }
        }
        
        found_roads.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        
        debug!("Found {} roads near target point:", found_roads.len());
        for (id, name, highway, distance) in found_roads.iter().take(10) {
            debug!("  Way {}: '{}' ({}), distance: {:.6}", id, name, highway, distance);
        }
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

#[derive(Debug, Clone)]
struct DataBounds {
    bounds: VisibleBounds,
    median_lat: f64,
    median_lon: f64,
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
                info!("Rectangle zoom selection completed");
            }
        }
        
        // Cancel selection on right click
        if response.secondary_clicked() {
            self.selection_rect = None;
            self.selection_mode = false;
            debug!("Rectangle zoom selection cancelled");
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
    
    /// Draw a highlight around the selected element
    fn draw_selection_highlight(&self, ui: &mut Ui, rect: Rect, map_data: &MapData) {
        if let Some(selected) = &self.selected_element {
            let painter = ui.painter_at(rect);
            let highlight_color = Color32::from_rgb(255, 100, 0); // Orange highlight
            let highlight_width = 3.0;
            
            match selected.element_type {
                ElementType::Way => {
                    if let Some(way) = map_data.ways.get(&selected.element_id) {
                        self.draw_way_highlight(&painter, rect, way, map_data, highlight_color, highlight_width);
                    }
                }
                ElementType::Node => {
                    if let Some(node) = map_data.nodes.get(&selected.element_id) {
                        self.draw_node_highlight(&painter, rect, node, highlight_color);
                    }
                }
                ElementType::Relation => {
                    // TODO: Implement relation highlighting if needed
                }
            }
        }
    }
    
    /// Draw highlight for a selected way
    fn draw_way_highlight(&self, painter: &egui::Painter, rect: Rect, way: &crate::core::Way, map_data: &MapData, color: Color32, width: f32) {
        if way.nodes.len() < 2 {
            return;
        }
        
        let mut screen_points = Vec::new();
        
        // Convert way nodes to screen coordinates
        for &node_id in &way.nodes {
            if let Some(node) = map_data.nodes.get(&node_id) {
                let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
                screen_points.push(screen_pos);
            }
        }
        
        if screen_points.len() < 2 {
            return;
        }
        
        // Draw highlight lines
        for i in 0..screen_points.len() - 1 {
            painter.line_segment(
                [screen_points[i], screen_points[i + 1]],
                egui::Stroke::new(width, color),
            );
        }
        
        // If it's a closed way (polygon), close it
        if screen_points.len() > 2 && way.nodes.first() == way.nodes.last() {
            // Already closed by the loop above
        }
    }
    
    /// Draw highlight for a selected node
    fn draw_node_highlight(&self, painter: &egui::Painter, rect: Rect, node: &crate::core::Node, color: Color32) {
        let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
        let radius = 8.0;
        
        // Draw a circle highlight around the node
        painter.circle_stroke(screen_pos, radius, egui::Stroke::new(3.0, color));
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
                    // Use StyleManager for water colors
                    let style = style_manager.get_current_style();
                    let water_color = Self::hex_to_rgb(&style.water.color);
                    let fill_color = Color32::from_rgba_unmultiplied(
                        water_color.0, 
                        water_color.1, 
                        water_color.2, 
                        (255.0 * style.water.opacity) as u8
                    );
                    
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        fill_color,
                        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(
                            water_color.0.saturating_sub(30), 
                            water_color.1.saturating_sub(30), 
                            water_color.2.saturating_sub(30), 
                            255
                        )),
                    ));
                }
            }
        }
    }
    
    fn draw_landuse_areas(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        let style = style_manager.get_current_style();
        
        for way in map_data.ways.values() {
            if !self.way_intersects_bounds(way, map_data, visible_bounds) || !way.is_closed {
                continue;
            }
            
            // Check if this way has landuse, leisure, or natural tags we care about
            let mut should_draw = false;
            let mut fill_color = Color32::TRANSPARENT;
            let stroke_color = Color32::from_rgb(200, 200, 200); // Default light stroke
            
            // Use StyleManager for landuse colors
            if let Some(landuse) = way.tags.get("landuse") {
                if let Some(color_str) = style.get_landuse_color(landuse) {
                    let (r, g, b) = Self::hex_to_rgb(color_str);
                    fill_color = Color32::from_rgb(r, g, b);
                    should_draw = true;
                }
            } else if let Some(leisure) = way.tags.get("leisure") {
                if let Some(color_str) = style.get_leisure_color(leisure) {
                    let (r, g, b) = Self::hex_to_rgb(color_str);
                    fill_color = Color32::from_rgb(r, g, b);
                    should_draw = true;
                }
            } else if let Some(natural) = way.tags.get("natural") {
                if let Some(color_str) = style.get_natural_color(natural) {
                    let (r, g, b) = Self::hex_to_rgb(color_str);
                    fill_color = Color32::from_rgb(r, g, b);
                    should_draw = true;
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
                    // Use style from StyleManager
                    let ((r, g, b), stroke_width) = self.get_way_style(way, style_manager);
                    let building_color = Color32::from_rgb(r, g, b);
                    let building_stroke = Color32::from_rgb(r.saturating_sub(28), g.saturating_sub(28), b.saturating_sub(28));
                    
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
                        egui::Stroke::new(stroke_width, building_stroke),
                    ));
                }
            }
        }
    }
    
    fn draw_road_casings(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        let mut total_roads = 0;
        let mut filtered_roads = 0;
        let mut rendered_roads = 0;
        
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                total_roads += 1;
                
                if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                    filtered_roads += 1;
                    
                    // Debug: Check for specific roads
                    if let Some(name) = way.tags.get("name") {
                        if name.to_lowercase().contains("bezons") || name.to_lowercase().contains("bernanos") {
                            debug!("'{}' ({}) is OUTSIDE visible bounds", name, highway);
                            
                            // Show coordinates and bounds
                            let mut coords = Vec::new();
                            for &node_id in &way.nodes {
                                if let Some(node) = map_data.nodes.get(&node_id) {
                                    coords.push((node.lat, node.lon));
                                }
                            }
                            
                            if !coords.is_empty() {
                                debug!("  First node: lat={:.6}, lon={:.6}", coords[0].0, coords[0].1);
                                debug!("  Last node: lat={:.6}, lon={:.6}", coords[coords.len()-1].0, coords[coords.len()-1].1);
                            }
                            
                            debug!("  Visible bounds: lat {:.6} to {:.6}, lon {:.6} to {:.6}", 
                                     visible_bounds.min_lat, visible_bounds.max_lat, 
                                     visible_bounds.min_lon, visible_bounds.max_lon);
                        }
                    }
                    continue;
                }
                
                // Debug: Check if this is a road we want to track
                if let Some(name) = way.tags.get("name") {
                    if name.to_lowercase().contains("bezons") || name.to_lowercase().contains("bernanos") {
                        debug!("'{}' ({}) is INSIDE visible bounds and will be rendered", name, highway);
                    }
                }
                
                let (casing_width, casing_color) = {
                    let style = style_manager.get_current_style();
                    let (_, _, border_color, border_width) = style.get_road_style(highway);
                    
                    if border_width > 0.0 && !border_color.is_empty() {
                        let (r, g, b) = Self::hex_to_rgb(border_color);
                        (border_width, Color32::from_rgb(r, g, b))
                    } else {
                        (0.0, Color32::TRANSPARENT)
                    }
                };
                
                if casing_width > 0.0 {
                    let points: Vec<Pos2> = way.nodes
                        .iter()
                        .filter_map(|&node_id| map_data.nodes.get(&node_id))
                        .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                        .collect();
                    
                    if points.len() >= 2 {
                        rendered_roads += 1;
                        
                        // Debug: Check if this is a specific road
                        if let Some(name) = way.tags.get("name") {
                            if name.to_lowercase().contains("bezons") || name.to_lowercase().contains("bernanos") {
                                debug!("Rendering casing for '{}' - highway: {}, width: {:.1}, color: {:?}", 
                                         name, highway, casing_width, casing_color);
                                debug!("  {} screen points: {:?}", points.len(), 
                                         points.iter().take(3).collect::<Vec<_>>());
                            }
                        }
                        
                        painter.add(egui::Shape::line(
                            points,
                            egui::Stroke::new(casing_width, casing_color),
                        ));
                    }
                }
            }
        }
        
        debug!("Road casings - Total: {}, Filtered: {}, Rendered: {}", 
                 total_roads, filtered_roads, rendered_roads);
    }
    
    fn draw_road_fills(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        let mut total_roads = 0;
        let mut filtered_roads = 0;
        let mut rendered_roads = 0;
        
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                total_roads += 1;
                
                if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                    filtered_roads += 1;
                    continue;
                }
                
                // Use style from StyleManager instead of hardcoded colors
                let ((r, g, b), width) = self.get_way_style(way, style_manager);
                let color = Color32::from_rgb(r, g, b);
                
                let points: Vec<Pos2> = way.nodes
                    .iter()
                    .filter_map(|&node_id| map_data.nodes.get(&node_id))
                    .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                    .collect();
                
                if points.len() >= 2 {
                    rendered_roads += 1;
                    
                    // Debug: Check if this is a specific road
                    if let Some(name) = way.tags.get("name") {
                        if name.to_lowercase().contains("bezons") || name.to_lowercase().contains("bernanos") {
                            debug!("Rendering fill for '{}' - highway: {}, width: {:.1}, color: {:?}", 
                                     name, highway, width, color);
                        }
                    }
                    
                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new(width, color),
                    ));
                }
            }
        }
        
        debug!("Road fills - Total: {}, Filtered: {}, Rendered: {}", 
                 total_roads, filtered_roads, rendered_roads);
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
                    // Use StyleManager for railway styling
                    let style = style_manager.get_current_style();
                    let rail_color = Self::hex_to_rgb(&style.railway.rail_color);
                    
                    // Draw railway as dashed line
                    painter.add(egui::Shape::dashed_line(
                        &points,
                        egui::Stroke::new(style.railway.rail_width, Color32::from_rgb(rail_color.0, rail_color.1, rail_color.2)),
                        10.0,
                        5.0,
                    ));
                }
            }
        }
    }
    
    fn draw_pois(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        // Draw POIs from nodes with amenity, shop, or other POI tags
        for node in map_data.nodes.values() {
            // Check if node is within visible bounds
            if !self.node_intersects_bounds(node, visible_bounds) {
                continue;
            }
            
            // Check if this node is a POI
            if let Some(poi_type) = self.get_poi_type(node) {
                let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
                
                // Get style for this POI type
                let (color_str, radius) = style_manager.get_current_style().get_poi_style(&poi_type);
                let color = Self::hex_to_rgb(color_str);
                let poi_color = Color32::from_rgb(color.0, color.1, color.2);
                
                // Draw POI as a circle
                painter.circle_filled(screen_pos, radius, poi_color);
                
                // Add a subtle border
                painter.circle_stroke(screen_pos, radius, egui::Stroke::new(0.5, Color32::from_rgb(0, 0, 0)));
                
                // Optionally draw POI name if available and zoom level is high enough
                if self.viewport.scale > 50.0 {
                    if let Some(name) = node.tags.get("name") {
                        let label_pos = Pos2::new(screen_pos.x, screen_pos.y - radius - 2.0);
                        painter.text(
                            label_pos,
                            egui::Align2::CENTER_BOTTOM,
                            name,
                            egui::FontId::proportional(9.0),
                            Color32::BLACK,
                        );
                    }
                }
            }
        }
    }
    
    /// Determine if a node is a POI and return its type
    fn get_poi_type(&self, node: &crate::core::Node) -> Option<String> {
        // Check amenity tags first (restaurants, cafes, hospitals, etc.)
        if let Some(amenity) = node.tags.get("amenity") {
            return Some(amenity.clone());
        }
        
        // Check shop tags
        if let Some(shop) = node.tags.get("shop") {
            return Some(format!("shop_{}", shop));
        }
        
        // Check tourism tags
        if let Some(tourism) = node.tags.get("tourism") {
            return Some(format!("tourism_{}", tourism));
        }
        
        // Check leisure tags
        if let Some(leisure) = node.tags.get("leisure") {
            return Some(format!("leisure_{}", leisure));
        }
        
        // Check office tags
        if let Some(office) = node.tags.get("office") {
            return Some(format!("office_{}", office));
        }
        
        // Check healthcare tags
        if let Some(healthcare) = node.tags.get("healthcare") {
            return Some(format!("healthcare_{}", healthcare));
        }
        
        // Check public transport
        if node.tags.contains_key("public_transport") {
            return Some("public_transport".to_string());
        }
        
        // Check if it's a place (city, town, village, etc.)
        if let Some(place) = node.tags.get("place") {
            return Some(format!("place_{}", place));
        }
        
        None
    }
    
    /// Check if a node intersects with visible bounds
    fn node_intersects_bounds(&self, node: &crate::core::Node, visible_bounds: &VisibleBounds) -> bool {
        node.lat >= visible_bounds.min_lat 
            && node.lat <= visible_bounds.max_lat
            && node.lon >= visible_bounds.min_lon 
            && node.lon <= visible_bounds.max_lon
    }
    
    // Points of interest drawing has been disabled - no individual nodes will be drawn
    
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
        
        // Draw node labels (POI names) - DISABLED
        // No longer drawing individual OSM nodes or their labels
    }
    
    // Legacy hardcoded style functions - replaced with StyleManager
    // These remain for backward compatibility but should not be used
    #[allow(dead_code)]
    
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
    
    /// Handle element selection when clicking on the map in Select mode
    fn handle_element_selection(&mut self, click_pos: Pos2, rect: Rect, map_data: &Option<MapData>) {
        if let Some(data) = map_data {
            let (click_lon, click_lat) = self.screen_to_map(click_pos, rect);
            let tolerance = 10.0 / self.viewport.scale; // Click tolerance in map units
            
            debug!("Element selection at: {:.6}, {:.6} (tolerance: {:.6})", click_lon, click_lat, tolerance);
            
            // Find the closest element to the click position
            let mut closest_element: Option<SelectedElement> = None;
            let mut closest_distance = f64::INFINITY;
            
            // Check ways (roads, buildings, areas)
            for way in data.ways.values() {
                if let Some(distance) = self.calculate_way_distance(way, data, click_lon, click_lat) {
                    if distance < tolerance && distance < closest_distance {
                        closest_distance = distance;
                        
                        // Determine the style information
                        let style_info = self.determine_style_info(&way.tags);
                        
                        closest_element = Some(SelectedElement {
                            element_type: ElementType::Way,
                            element_id: way.id,
                            tags: way.tags.clone(),
                            style_info,
                        });
                    }
                }
            }
            
            // Check nodes (POIs, etc.) if no way was found nearby
            if closest_element.is_none() {
                for node in data.nodes.values() {
                    let node_distance = ((node.lon - click_lon).powi(2) + (node.lat - click_lat).powi(2)).sqrt();
                    if node_distance < tolerance && node_distance < closest_distance {
                        closest_distance = node_distance;
                        
                        // Only select nodes that have interesting tags (POIs, etc.)
                        if self.is_selectable_node(node) {
                            let style_info = self.determine_style_info(&node.tags);
                            
                            closest_element = Some(SelectedElement {
                                element_type: ElementType::Node,
                                element_id: node.id,
                                tags: node.tags.clone(),
                                style_info,
                            });
                        }
                    }
                }
            }
            
            // Update selection
            if let Some(element) = closest_element {
                info!("Selected {} {} with tags: {:?}", 
                      match element.element_type {
                          ElementType::Way => "way",
                          ElementType::Node => "node", 
                          ElementType::Relation => "relation",
                      },
                      element.element_id, 
                      element.tags);
                      
                debug!("Style info: category={}, subcategory={}, toml_section={}", 
                       element.style_info.category, 
                       element.style_info.subcategory,
                       element.style_info.toml_section);
                       
                self.selected_element = Some(element);
            } else {
                debug!("No selectable element found near click position");
                self.selected_element = None;
            }
        }
    }
    
    /// Calculate the distance from a point to a way (line or polygon)
    fn calculate_way_distance(&self, way: &crate::core::Way, map_data: &MapData, lon: f64, lat: f64) -> Option<f64> {
        if way.nodes.len() < 2 {
            return None;
        }
        
        let mut min_distance = f64::INFINITY;
        
        // Convert way nodes to coordinates
        let coords: Vec<(f64, f64)> = way.nodes.iter()
            .filter_map(|&node_id| {
                map_data.nodes.get(&node_id).map(|node| (node.lon, node.lat))
            })
            .collect();
            
        if coords.len() < 2 {
            return None;
        }
        
        // Calculate distance to each line segment
        for i in 0..coords.len() - 1 {
            let (x1, y1) = coords[i];
            let (x2, y2) = coords[i + 1];
            let distance = self.point_to_line_distance(lon, lat, x1, y1, x2, y2);
            min_distance = min_distance.min(distance);
        }
        
        // If it's a closed way (polygon), also check the closing segment
        if coords.len() > 2 && coords[0] == *coords.last().unwrap() {
            // Already handled by the loop above since it's a closed way
        }
        
        Some(min_distance)
    }
    
    /// Calculate distance from a point to a line segment
    fn point_to_line_distance(&self, px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let line_length_sq = (x2 - x1).powi(2) + (y2 - y1).powi(2);
        
        if line_length_sq == 0.0 {
            // Line segment is actually a point
            return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
        }
        
        // Calculate the parameter t that represents the closest point on the line segment
        let t = ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / line_length_sq;
        let t = t.clamp(0.0, 1.0); // Clamp to line segment
        
        // Find the closest point on the line segment
        let closest_x = x1 + t * (x2 - x1);
        let closest_y = y1 + t * (y2 - y1);
        
        // Return distance to closest point
        ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
    }
    
    /// Determine if a node is selectable (has interesting tags)
    fn is_selectable_node(&self, node: &crate::core::Node) -> bool {
        // Select nodes that have tags indicating they're POIs or features
        node.tags.contains_key("amenity") ||
        node.tags.contains_key("shop") ||
        node.tags.contains_key("tourism") ||
        node.tags.contains_key("leisure") ||
        node.tags.contains_key("office") ||
        node.tags.contains_key("craft") ||
        node.tags.contains_key("emergency") ||
        node.tags.contains_key("historic") ||
        node.tags.contains_key("natural") ||
        node.tags.contains_key("barrier") ||
        (node.tags.contains_key("name") && node.tags.len() > 1)
    }
    
    /// Determine style information for an element based on its tags
    fn determine_style_info(&self, tags: &HashMap<String, String>) -> StyleInfo {
        // Determine category and subcategory based on tags, matching actual TOML structure
        if let Some(highway) = tags.get("highway") {
            StyleInfo {
                category: "road".to_string(),
                subcategory: highway.clone(),
                toml_section: format!("roads.{}", highway),
            }
        } else if tags.get("building").is_some() {
            StyleInfo {
                category: "building".to_string(),
                subcategory: tags.get("building").unwrap_or(&"yes".to_string()).clone(),
                toml_section: "buildings".to_string(),
            }
        } else if let Some(natural) = tags.get("natural") {
            StyleInfo {
                category: "natural".to_string(),
                subcategory: natural.clone(),
                toml_section: "natural".to_string(),
            }
        } else if let Some(landuse) = tags.get("landuse") {
            StyleInfo {
                category: "landuse".to_string(),
                subcategory: landuse.clone(),
                toml_section: "landuse".to_string(),
            }
        } else if let Some(leisure) = tags.get("leisure") {
            StyleInfo {
                category: "leisure".to_string(),
                subcategory: leisure.clone(),
                toml_section: "leisure".to_string(),
            }
        } else if tags.get("natural").map(|n| n == "water").unwrap_or(false) || tags.get("waterway").is_some() {
            StyleInfo {
                category: "water".to_string(),
                subcategory: tags.get("waterway").unwrap_or(&"water".to_string()).clone(),
                toml_section: "water".to_string(),
            }
        } else if let Some(amenity) = tags.get("amenity") {
            // Map amenity to POI section
            StyleInfo {
                category: "poi".to_string(),
                subcategory: amenity.clone(),
                toml_section: format!("pois.{}", amenity),
            }
        } else if let Some(shop) = tags.get("shop") {
            // Map shop to POI section
            StyleInfo {
                category: "poi".to_string(),
                subcategory: format!("shop_{}", shop),
                toml_section: format!("pois.shop_{}", shop),
            }
        } else if let Some(tourism) = tags.get("tourism") {
            // Map tourism to POI section
            StyleInfo {
                category: "poi".to_string(),
                subcategory: format!("tourism_{}", tourism),
                toml_section: format!("pois.tourism_{}", tourism),
            }
        } else if let Some(railway) = tags.get("railway") {
            StyleInfo {
                category: "railway".to_string(),
                subcategory: railway.clone(),
                toml_section: "railway".to_string(),
            }
        } else if let Some(aeroway) = tags.get("aeroway") {
            StyleInfo {
                category: "aeroway".to_string(),
                subcategory: aeroway.clone(),
                toml_section: "aeroway".to_string(),
            }
        } else if tags.get("admin_level").is_some() {
            StyleInfo {
                category: "boundary".to_string(),
                subcategory: "administrative".to_string(),
                toml_section: "boundaries".to_string(),
            }
        } else {
            // Generic element
            StyleInfo {
                category: "unknown".to_string(),
                subcategory: "generic".to_string(),
                toml_section: "unknown".to_string(),
            }
        }
    }
}
