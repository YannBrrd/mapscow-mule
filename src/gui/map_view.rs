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
        let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag());
        
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
    
    fn handle_input(&mut self, ui: &mut Ui, response: &Response, rect: Rect) {
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
        
        // Handle mouse wheel for zooming
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        if scroll_delta.y != 0.0 {
            let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 1.0 / 1.1 };
            
            // Zoom towards mouse position if available
            if let Some(mouse_pos) = response.hover_pos() {
                let rel_x = (mouse_pos.x - rect.center().x) as f64;
                let rel_y = -(mouse_pos.y - rect.center().y) as f64; // Flip Y
                
                // Convert to map coordinates
                let map_x = self.viewport.center_x + rel_x / self.viewport.scale;
                let map_y = self.viewport.center_y + rel_y / self.viewport.scale;
                
                // Apply zoom
                self.viewport.scale *= zoom_factor;
                
                // Adjust center to zoom towards mouse position
                self.viewport.center_x = map_x - rel_x / self.viewport.scale;
                self.viewport.center_y = map_y - rel_y / self.viewport.scale;
            } else {
                // Simple zoom at center
                self.viewport.scale *= zoom_factor;
            }
            
            // Clamp zoom level to allow for detailed street-level viewing
            self.viewport.scale = self.viewport.scale.clamp(0.001, 50000.0);
        }
    }
    
    fn draw_map(&self, ui: &mut Ui, rect: Rect, map_data: &Option<MapData>, renderer: &MapRenderer, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        // Draw background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(240, 248, 255));
        
        if let Some(data) = map_data {
            // Calculate visible bounds
            let visible_bounds = self.calculate_visible_bounds(rect);
            
            // Debug: Draw some info about the data
            let debug_text = format!(
                "Nodes: {}, Ways: {}\nScale: {:.2}, Center: ({:.6}, {:.6})",
                data.nodes.len(),
                data.ways.len(),
                self.viewport.scale,
                self.viewport.center_x,
                self.viewport.center_y
            );
            painter.text(
                rect.min + egui::Vec2::new(10.0, 50.0),
                egui::Align2::LEFT_TOP,
                debug_text,
                egui::FontId::monospace(10.0),
                Color32::RED,
            );
            
            // Draw map features
            self.draw_ways(ui, rect, data, &visible_bounds, style_manager);
            self.draw_nodes(ui, rect, data, &visible_bounds, style_manager);
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
        
        // Draw viewport info
        self.draw_viewport_info(ui, rect);
    }
    
    fn draw_ways(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        for way in map_data.ways.values() {
            // Check if way is potentially visible
            if !self.way_intersects_bounds(way, map_data, visible_bounds) {
                continue;
            }
            
            let points: Vec<Pos2> = way.nodes
                .iter()
                .filter_map(|&node_id| map_data.nodes.get(&node_id))
                .map(|node| self.map_to_screen(node.lon, node.lat, rect))
                .collect();
            
            if points.len() < 2 {
                continue;
            }
            
            // Determine drawing style based on tags and stylesheet
            let (color, width) = self.get_way_style(way, style_manager);
            
            // Draw the way
            if way.is_closed && points.len() > 2 {
                // Draw as filled polygon
                painter.add(egui::Shape::convex_polygon(
                    points,
                    Color32::from_rgba_unmultiplied(color.0, color.1, color.2, 100),
                    egui::Stroke::new(width, Color32::from_rgb(color.0, color.1, color.2)),
                ));
            } else {
                // Draw as line
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(width, Color32::from_rgb(color.0, color.1, color.2)),
                ));
            }
        }
    }
    
    fn draw_nodes(&self, ui: &mut Ui, rect: Rect, map_data: &MapData, visible_bounds: &VisibleBounds, style_manager: &StyleManager) {
        let painter = ui.painter_at(rect);
        
        let mut nodes_drawn = 0;
        for node in map_data.nodes.values() {
            // Draw ALL nodes for debugging, not just ones with tags
            // if node.tags.is_empty() {
            //     continue;
            // }
            
            // Check if node is visible
            if !self.point_in_bounds(node.lon, node.lat, visible_bounds) {
                continue;
            }
            
            nodes_drawn += 1;
            let screen_pos = self.map_to_screen(node.lon, node.lat, rect);
            let (color, size) = self.get_node_style(node, style_manager);
            
            // Draw node as circle
            painter.circle_filled(
                screen_pos,
                size,
                Color32::from_rgb(color.0, color.1, color.2),
            );
            
            // Draw name if available and zoom level is high enough
            if self.viewport.scale > 100.0 {
                if let Some(name) = node.tags.get("name") {
                    painter.text(
                        screen_pos + Vec2::new(0.0, size + 2.0),
                        egui::Align2::CENTER_TOP,
                        name,
                        egui::FontId::proportional(10.0),
                        Color32::BLACK,
                    );
                }
            }
        }
    }
    
    fn draw_viewport_info(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        
        // Draw zoom level and coordinates in corner
        let info_text = format!(
            "Zoom: {:.1}x\nCenter: {:.6}, {:.6}",
            self.viewport.scale,
            self.viewport.center_x,
            self.viewport.center_y
        );
        
        let text_pos = rect.min + Vec2::new(10.0, 10.0);
        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            info_text,
            egui::FontId::monospace(10.0),
            Color32::BLACK,
        );
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
                    
                    // Clamp the scale to allow for close zoom levels
                    // For local maps (few km), we want scales from hundreds to tens of thousands for street details
                    self.viewport.scale = self.viewport.scale.clamp(50.0, 50000.0);
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
        if response.clicked() {
            // Start new selection
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                // Ensure mouse is within the map area
                if rect.contains(mouse_pos) {
                    self.selection_rect = Some(SelectionRect {
                        start_pos: mouse_pos,
                        current_pos: mouse_pos,
                        has_been_dragged: false,
                    });
                }
            }
        } else if response.dragged() {
            // Update selection rectangle
            if let (Some(ref mut selection), Some(mouse_pos)) = (&mut self.selection_rect, response.interact_pointer_pos()) {
                // Mark that we've actually dragged
                selection.has_been_dragged = true;
                
                // Clamp mouse position to map bounds
                let clamped_pos = Pos2::new(
                    mouse_pos.x.clamp(rect.min.x, rect.max.x),
                    mouse_pos.y.clamp(rect.min.y, rect.max.y)
                );
                selection.current_pos = clamped_pos;
            }
        } else if response.drag_stopped() {
            // Complete selection and zoom to area - but only if we actually dragged
            if let Some(selection) = self.selection_rect.take() {
                if selection.has_been_dragged {
                    self.zoom_to_selection(&selection, rect);
                    self.selection_mode = false; // Exit selection mode after zoom
                    println!("Rectangle zoom selection completed");
                } else {
                    // Single click without drag - do nothing
                    println!("Single click detected, no zoom action");
                }
            }
        }
        
        // Update current position while hovering (even without dragging)
        if let (Some(ref mut selection), Some(mouse_pos)) = (&mut self.selection_rect, response.hover_pos()) {
            if rect.contains(mouse_pos) {
                selection.current_pos = mouse_pos;
            }
        }
        
        // Cancel selection on right click or escape
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
            
            // Clamp to reasonable zoom levels
            self.viewport.scale = self.viewport.scale.clamp(0.001, 50000.0);
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
            let instruction_text = if let Some(selection) = &self.selection_rect {
                if selection.has_been_dragged {
                    "Drag to select area, release to zoom"
                } else {
                    "Drag to select area (single clicks do nothing)"
                }
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
}
