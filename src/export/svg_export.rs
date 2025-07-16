use svg::node::element::{Group, Rectangle, Text, Path, Circle, Element};
use svg::node::element::path::Data;
use svg::Document;
use anyhow::Result;
use crate::rendering::{RenderedMap, RenderElement, ElementStyle};
use crate::core::MapData;
use crate::parsers::stylesheet::Color;
use crate::styles::loader::StyleManager;

pub struct SvgExporter {
    pub precision: usize,
    pub anti_aliasing: bool,
    pub layer_separation: bool,
    pub style_manager: StyleManager,
}

impl SvgExporter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            precision: 3,
            anti_aliasing: true,
            layer_separation: true,
            style_manager: StyleManager::new()?,
        })
    }

    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    pub fn with_anti_aliasing(mut self, enabled: bool) -> Self {
        self.anti_aliasing = enabled;
        self
    }

    pub fn with_layer_separation(mut self, enabled: bool) -> Self {
        self.layer_separation = enabled;
        self
    }

    pub fn export_with_data<P: AsRef<std::path::Path>>(
        &self,
        map_data: &MapData,
        output_path: P,
        width: u32,
        height: u32,
        center_lat: f64,
        center_lon: f64,
        scale: f64,
    ) -> Result<()> {
        let mut document = Document::new()
            .set("viewBox", (0, 0, width, height))
            .set("width", width)
            .set("height", height)
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("xmlns:inkscape", "http://www.inkscape.org/namespaces/inkscape")
            .set("xmlns:sodipodi", "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd");

        // Add Inkscape-specific metadata for better compatibility
        if self.anti_aliasing {
            document = document.set("shape-rendering", "geometricPrecision");
        }
        
        // Add Google Maps color scheme metadata
        let style = self.style_manager.get_current_style();
        document = document.set("style", format!("background-color:{}", style.background.color));

        // Create main group for all elements with Inkscape layer support
        let mut main_group = Group::new()
            .set("id", "map")
            .set("inkscape:label", "Map")
            .set("inkscape:groupmode", "layer");

        // Background color from style
        let background = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("fill", style.background.color.as_str())
            .set("stroke", "none");

        main_group = main_group.add(background);

        // Improved coordinate transformation with Web Mercator-like projection
        let to_svg_coords = |lat: f64, lon: f64| -> (f64, f64) {
            // Simple equirectangular projection with better scaling
            let _lat_rad = lat.to_radians();
            let center_lat_rad = center_lat.to_radians();
            
            // Apply Web Mercator-like scaling for better visual representation
            let y_scale = center_lat_rad.cos();
            
            let x = (width as f64 / 2.0) + (lon - center_lon) * scale * y_scale;
            let y = (height as f64 / 2.0) - (lat - center_lat) * scale;
            
            (self.round_value(x), self.round_value(y))
        };

        // Create separate layer groups for better organization (Google Maps style)
        let mut water_group = Group::new()
            .set("id", "water")
            .set("inkscape:label", "Water")
            .set("inkscape:groupmode", "layer");
        let mut landuse_group = Group::new()
            .set("id", "landuse")
            .set("inkscape:label", "Land Use")
            .set("inkscape:groupmode", "layer");
        let mut aeroway_group = Group::new()
            .set("id", "aeroway")
            .set("inkscape:label", "Aeroway")
            .set("inkscape:groupmode", "layer");
        let mut buildings_group = Group::new()
            .set("id", "buildings")
            .set("inkscape:label", "Buildings")
            .set("inkscape:groupmode", "layer");
        let mut roads_group = Group::new()
            .set("id", "roads")
            .set("inkscape:label", "Roads")
            .set("inkscape:groupmode", "layer");
        let mut railway_group = Group::new()
            .set("id", "railway")
            .set("inkscape:label", "Railway")
            .set("inkscape:groupmode", "layer");
        let mut boundaries_group = Group::new()
            .set("id", "boundaries")
            .set("inkscape:label", "Boundaries")
            .set("inkscape:groupmode", "layer");
        let mut pois_group = Group::new()
            .set("id", "pois")
            .set("inkscape:label", "Points of Interest")
            .set("inkscape:groupmode", "layer");
        let mut labels_group = Group::new()
            .set("id", "labels")
            .set("inkscape:label", "Labels")
            .set("inkscape:groupmode", "layer");

        // Draw water bodies using style
        for way in map_data.ways.values() {
            if self.is_water_feature(way) {
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let water_path = Path::new()
                        .set("d", path_data)
                        .set("fill", style.water.color.as_str())
                        .set("stroke", "none")
                        .set("opacity", style.water.opacity);
                    water_group = water_group.add(water_path);
                }
            }
        }

        // Draw land use areas using style
        for way in map_data.ways.values() {
            if let Some(landuse) = way.tags.get("landuse") {
                if let Some(fill_color) = style.get_landuse_color(landuse) {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        let area_path = Path::new()
                            .set("d", path_data)
                            .set("fill", fill_color)
                            .set("stroke", "none")
                            .set("opacity", 1.0);
                        landuse_group = landuse_group.add(area_path);
                    }
                }
            }
            
            // Handle leisure areas (parks, etc.)
            if let Some(leisure) = way.tags.get("leisure") {
                if let Some(fill_color) = style.get_leisure_color(leisure) {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        let area_path = Path::new()
                            .set("d", path_data)
                            .set("fill", fill_color)
                            .set("stroke", "none")
                            .set("opacity", 1.0);
                        landuse_group = landuse_group.add(area_path);
                    }
                }
            }
            
            // Handle natural areas
            if let Some(natural) = way.tags.get("natural") {
                if let Some(fill_color) = style.get_natural_color(natural) {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        let area_path = Path::new()
                            .set("d", path_data)
                            .set("fill", fill_color)
                            .set("stroke", "none")
                            .set("opacity", 1.0);
                        landuse_group = landuse_group.add(area_path);
                    }
                }
            }
        }
        
        // Draw aeroway areas using style
        for way in map_data.ways.values() {
            if way.tags.contains_key("aeroway") {
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let aeroway_path = Path::new()
                        .set("d", path_data)
                        .set("fill", style.aeroway.default.as_str())
                        .set("stroke", "none")
                        .set("opacity", 1.0);
                    aeroway_group = aeroway_group.add(aeroway_path);
                }
            }
        }

        // Draw buildings using style
        for way in map_data.ways.values() {
            if way.tags.contains_key("building") {
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let building_path = Path::new()
                        .set("d", path_data)
                        .set("fill", style.buildings.fill.as_str())
                        .set("stroke", style.buildings.stroke.as_str())
                        .set("stroke-width", style.buildings.stroke_width)
                        .set("opacity", 1.0);
                    buildings_group = buildings_group.add(building_path);
                }
            }
        }
        
        // Draw railways using style
        for way in map_data.ways.values() {
            if let Some(railway) = way.tags.get("railway") {
                if railway == "rail" {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        // Railway main line
                        let railway_path = Path::new()
                            .set("d", path_data.clone())
                            .set("fill", "none")
                            .set("stroke", style.railway.rail_color.as_str())
                            .set("stroke-width", style.railway.rail_width)
                            .set("stroke-linecap", "round");
                        railway_group = railway_group.add(railway_path);
                        
                        // Railway dashes
                        let railway_dashes = Path::new()
                            .set("d", path_data)
                            .set("fill", "none")
                            .set("stroke", style.railway.rail_dash_color.as_str())
                            .set("stroke-width", style.railway.rail_dash_width)
                            .set("stroke-dasharray", style.railway.rail_dash_pattern.as_str())
                            .set("stroke-linecap", "round");
                        railway_group = railway_group.add(railway_dashes);
                    }
                }
            }
        }

        // Draw roads with styling from config
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                let (stroke_color, stroke_width, border_color, border_width) = style.get_road_style(highway);
                
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    // Draw road border first (if exists)
                    if !border_color.is_empty() && border_width > 0.0 {
                        let border_path = Path::new()
                            .set("d", path_data.clone())
                            .set("fill", "none")
                            .set("stroke", border_color)
                            .set("stroke-width", stroke_width + border_width * 2.0)
                            .set("stroke-linecap", "round")
                            .set("stroke-linejoin", "round");
                        roads_group = roads_group.add(border_path);
                    }
                    
                    // Draw main road
                    let road_path = Path::new()
                        .set("d", path_data)
                        .set("fill", "none")
                        .set("stroke", stroke_color)
                        .set("stroke-width", stroke_width)
                        .set("stroke-linecap", "round")
                        .set("stroke-linejoin", "round");
                    roads_group = roads_group.add(road_path);

                    // Add road name labels for major roads (moved to labels group)
                    if let Some(name) = way.tags.get("name") {
                        if self.should_label_road(highway) && !name.trim().is_empty() {
                            if let Some(label_position) = self.calculate_road_label_position(way, map_data, &to_svg_coords) {
                                let font_size = style.get_road_label_font_size(highway);
                                let road_label = Text::new(name)
                                    .set("x", label_position.0)
                                    .set("y", label_position.1)
                                    .set("text-anchor", "middle")
                                    .set("dominant-baseline", "central")
                                    .set("font-family", style.labels.font_family.as_str())
                                    .set("font-size", font_size)
                                    .set("font-weight", "normal")
                                    .set("fill", "#000000")
                                    .set("stroke", style.labels.road_label_stroke.as_str())
                                    .set("stroke-width", style.labels.road_label_stroke_width)
                                    .set("stroke-linejoin", "round")
                                    .set("paint-order", "stroke fill")
                                    .set("opacity", 1.0);
                                labels_group = labels_group.add(road_label);
                            }
                        }
                    }
                }
            }
        }

        // Draw boundaries using style
        for way in map_data.ways.values() {
            if let Some(boundary) = way.tags.get("boundary") {
                if boundary == "administrative" {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        let boundary_path = Path::new()
                            .set("d", path_data)
                            .set("fill", "none")
                            .set("stroke", style.boundaries.administrative_color.as_str())
                            .set("stroke-width", style.boundaries.administrative_width)
                            .set("stroke-dasharray", style.boundaries.administrative_dash.as_str())
                            .set("opacity", style.boundaries.administrative_opacity);
                        boundaries_group = boundaries_group.add(boundary_path);
                    }
                }
            }
        }

        // Draw POIs with styling from config
        for node in map_data.nodes.values() {
            if let Some(amenity) = node.tags.get("amenity") {
                let (x, y) = to_svg_coords(node.lat, node.lon);
                if x >= 0.0 && x <= width as f64 && y >= 0.0 && y <= height as f64 {
                    let (color, radius) = style.get_poi_style(amenity);
                    
                    let poi_circle = Circle::new()
                        .set("cx", x)
                        .set("cy", y)
                        .set("r", radius)
                        .set("fill", color)
                        .set("stroke", "#ffffff")
                        .set("stroke-width", 1.5)
                        .set("opacity", 1.0);
                    pois_group = pois_group.add(poi_circle);
                    
                    // Add labels for important POIs
                    if self.is_important_poi(amenity) {
                        if let Some(name) = node.tags.get("name") {
                            let label = Text::new(name)
                                .set("x", x + radius as f64 + 8.0)
                                .set("y", y)
                                .set("text-anchor", "start")
                                .set("dominant-baseline", "central")
                                .set("font-family", style.labels.font_family.as_str())
                                .set("font-size", 10)
                                .set("font-weight", "normal")
                                .set("fill", "#333333")
                                .set("stroke", style.labels.poi_label_stroke.as_str())
                                .set("stroke-width", style.labels.poi_label_stroke_width)
                                .set("paint-order", "stroke fill");
                            labels_group = labels_group.add(label);
                        }
                    }
                }
            }
            
            // Handle place labels (cities, towns, etc.)
            if let Some(place) = node.tags.get("place") {
                let (x, y) = to_svg_coords(node.lat, node.lon);
                if x >= 0.0 && x <= width as f64 && y >= 0.0 && y <= height as f64 {
                    if let Some(name) = node.tags.get("name") {
                        let font_size = style.get_place_label_font_size(place);
                        let place_label = Text::new(name)
                            .set("x", x)
                            .set("y", y)
                            .set("text-anchor", "middle")
                            .set("dominant-baseline", "central")
                            .set("font-family", style.labels.font_family.as_str())
                            .set("font-size", font_size)
                            .set("font-weight", "bold")
                            .set("fill", "#000000")
                            .set("stroke", style.labels.place_label_stroke.as_str())
                            .set("stroke-width", style.labels.place_label_stroke_width)
                            .set("paint-order", "stroke fill");
                        labels_group = labels_group.add(place_label);
                    }
                }
            }
        }

        // Add all layer groups in proper Google Maps order (back to front)
        main_group = main_group.add(water_group);
        main_group = main_group.add(landuse_group);
        main_group = main_group.add(aeroway_group);
        main_group = main_group.add(buildings_group);
        main_group = main_group.add(railway_group);
        main_group = main_group.add(roads_group);
        main_group = main_group.add(boundaries_group);
        main_group = main_group.add(pois_group);
        main_group = main_group.add(labels_group);

        document = document.add(main_group);

        // Write to file using svg crate's save function
        svg::save(output_path, &document)?;
        Ok(())
    }

    fn is_water_feature(&self, way: &crate::core::Way) -> bool {
        if let Some(natural) = way.tags.get("natural") {
            matches!(natural.as_str(), "water" | "coastline")
        } else {
            way.tags.contains_key("waterway")
        }
    }

    fn is_important_poi(&self, amenity: &str) -> bool {
        matches!(amenity, "hospital" | "school" | "university" | "police" | "fire_station")
    }

    fn should_label_road(&self, highway: &str) -> bool {
        // Label more road types to ensure visibility
        matches!(highway, 
            "motorway" | "trunk" | "primary" | "secondary" | "tertiary" | 
            "motorway_link" | "trunk_link" | "primary_link" | "secondary_link" |
            "residential" | "unclassified" | "service" // Added more road types
        )
    }

    fn calculate_road_label_position<F>(&self, way: &crate::core::Way, map_data: &MapData, to_svg_coords: &F) -> Option<(f64, f64)>
    where
        F: Fn(f64, f64) -> (f64, f64),
    {
        if way.nodes.len() < 2 {
            return None;
        }

        // Find the middle segment of the road for label placement
        let mid_index = way.nodes.len() / 2;
        
        // Try to use the middle node, or average of two middle nodes
        if let Some(node_id) = way.nodes.get(mid_index) {
            if let Some(node) = map_data.nodes.get(node_id) {
                let coords = to_svg_coords(node.lat, node.lon);
                return Some(self.round_coords(coords));
            }
        }

        // Fallback: use first node if middle node is not available
        if let Some(node_id) = way.nodes.first() {
            if let Some(node) = map_data.nodes.get(node_id) {
                let coords = to_svg_coords(node.lat, node.lon);
                return Some(self.round_coords(coords));
            }
        }

        None
    }

    fn way_to_svg_path<F>(&self, way: &crate::core::Way, map_data: &MapData, to_svg_coords: &F) -> Option<Data>
    where
        F: Fn(f64, f64) -> (f64, f64),
    {
        if way.nodes.is_empty() {
            return None;
        }

        let mut data = Data::new();
        let mut first = true;
        let mut valid_points = 0;

        for node_id in &way.nodes {
            if let Some(node) = map_data.nodes.get(node_id) {
                let coords = to_svg_coords(node.lat, node.lon);
                let rounded_coords = self.round_coords(coords);
                
                if first {
                    data = data.move_to(rounded_coords);
                    first = false;
                } else {
                    data = data.line_to(rounded_coords);
                }
                valid_points += 1;
            }
        }

        // Only return valid paths
        if valid_points < 2 {
            return None;
        }

        // Close path if it's a closed way (area) and has enough points
        if way.is_closed && valid_points > 2 {
            data = data.close();
        }

        Some(data)
    }

    pub fn export<P: AsRef<std::path::Path>>(
        &self,
        rendered_map: &RenderedMap,
        output_path: P,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let mut document = Document::new()
            .set("viewBox", (0, 0, width, height))
            .set("width", width)
            .set("height", height)
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("xmlns:inkscape", "http://www.inkscape.org/namespaces/inkscape")
            .set("xmlns:sodipodi", "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd");

        // Add Inkscape-specific metadata for better compatibility
        if self.anti_aliasing {
            document = document.set("shape-rendering", "geometricPrecision");
        }

        // Create main group for all elements with Inkscape layer support
        let mut main_group = Group::new()
            .set("id", "map")
            .set("inkscape:label", "Map")
            .set("inkscape:groupmode", "layer");

        // Add Google Maps background
        let background = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("fill", "#F2EFE9");

        main_group = main_group.add(background);

        if self.layer_separation {
            // Render elements grouped by type for better organization
            main_group = self.render_by_layers(main_group, rendered_map)?;
        } else {
            // Render elements in z-order
            main_group = self.render_by_z_order(main_group, rendered_map)?;
        }

        document = document.add(main_group);

        // Write to file using svg crate's save function
        svg::save(output_path, &document)?;
        Ok(())
    }

    fn render_by_layers(&self, mut main_group: Group, rendered_map: &RenderedMap) -> Result<Group> {
        // Create separate groups for different element types with Inkscape layer support
        let mut polygon_group = Group::new()
            .set("id", "polygons")
            .set("inkscape:label", "Polygons")
            .set("inkscape:groupmode", "layer");
        let mut line_group = Group::new()
            .set("id", "lines")
            .set("inkscape:label", "Lines")
            .set("inkscape:groupmode", "layer");
        let mut circle_group = Group::new()
            .set("id", "circles")
            .set("inkscape:label", "Circles")
            .set("inkscape:groupmode", "layer");
        let mut text_group = Group::new()
            .set("id", "text")
            .set("inkscape:label", "Text")
            .set("inkscape:groupmode", "layer");

        for element in &rendered_map.elements {
            match element {
                RenderElement::Polygon { exterior, holes, style } => {
                    polygon_group = polygon_group.add(self.create_svg_polygon(exterior, holes, style)?);
                }
                RenderElement::Line { points, style } => {
                    line_group = line_group.add(self.create_svg_line(points, style)?);
                }
                RenderElement::Circle { center, radius, style } => {
                    circle_group = circle_group.add(self.create_svg_circle(*center, *radius, style)?);
                }
                RenderElement::Text { position, text, style } => {
                    text_group = text_group.add(self.create_svg_text(*position, text, style)?);
                }
            }
        }

        // Add groups in rendering order (back to front)
        main_group = main_group.add(polygon_group);
        main_group = main_group.add(line_group);
        main_group = main_group.add(circle_group);
        main_group = main_group.add(text_group);

        Ok(main_group)
    }

    fn render_by_z_order(&self, mut main_group: Group, rendered_map: &RenderedMap) -> Result<Group> {
        // For now, just render in order (could be enhanced with z-index sorting)
        for element in &rendered_map.elements {
            match element {
                RenderElement::Polygon { exterior, holes, style } => {
                    main_group = main_group.add(self.create_svg_polygon(exterior, holes, style)?);
                }
                RenderElement::Line { points, style } => {
                    main_group = main_group.add(self.create_svg_line(points, style)?);
                }
                RenderElement::Circle { center, radius, style } => {
                    main_group = main_group.add(self.create_svg_circle(*center, *radius, style)?);
                }
                RenderElement::Text { position, text, style } => {
                    main_group = main_group.add(self.create_svg_text(*position, text, style)?);
                }
            }
        }

        Ok(main_group)
    }

    fn create_svg_polygon(&self, exterior: &[(f64, f64)], holes: &[Vec<(f64, f64)>], style: &ElementStyle) -> Result<Element> {
        if exterior.is_empty() {
            return Ok(Group::new().into());
        }

        let mut path_data = Data::new();
        
        // Add exterior ring
        if let Some(&first_point) = exterior.first() {
            path_data = path_data.move_to(self.round_coords(first_point));
            for &point in exterior.iter().skip(1) {
                path_data = path_data.line_to(self.round_coords(point));
            }
            path_data = path_data.close();
        }

        // Add holes
        for hole in holes {
            if let Some(&first_point) = hole.first() {
                path_data = path_data.move_to(self.round_coords(first_point));
                for &point in hole.iter().skip(1) {
                    path_data = path_data.line_to(self.round_coords(point));
                }
                path_data = path_data.close();
            }
        }

        let mut path = Path::new().set("d", path_data);
        path = self.apply_style_to_path(path, style);
        
        Ok(path.into())
    }

    fn create_svg_line(&self, points: &[(f64, f64)], style: &ElementStyle) -> Result<Element> {
        if points.len() < 2 {
            return Ok(Group::new().into());
        }

        let mut path_data = Data::new();
        if let Some(&first_point) = points.first() {
            path_data = path_data.move_to(self.round_coords(first_point));
            for &point in points.iter().skip(1) {
                path_data = path_data.line_to(self.round_coords(point));
            }
        }

        let mut path = Path::new()
            .set("d", path_data)
            .set("fill", "none");
        
        path = self.apply_style_to_path(path, style);
        
        Ok(path.into())
    }

    fn create_svg_circle(&self, center: (f64, f64), radius: f64, style: &ElementStyle) -> Result<Element> {
        let center = self.round_coords(center);
        let mut circle = Circle::new()
            .set("cx", center.0)
            .set("cy", center.1)
            .set("r", self.round_value(radius));

        circle = self.apply_style_to_circle(circle, style);
        
        Ok(circle.into())
    }

    fn create_svg_text(&self, position: (f64, f64), text: &str, style: &ElementStyle) -> Result<Element> {
        let position = self.round_coords(position);
        let mut text_element = Text::new(text)
            .set("x", position.0)
            .set("y", position.1);

        text_element = self.apply_style_to_text(text_element, style);
        
        Ok(text_element.into())
    }

    fn apply_style_to_path(&self, mut path: Path, style: &ElementStyle) -> Path {
        if let Some(ref fill_color) = style.fill_color {
            path = path.set("fill", self.color_to_string(fill_color));
            if style.fill_opacity < 1.0 {
                path = path.set("fill-opacity", style.fill_opacity);
            }
        } else {
            path = path.set("fill", "none");
        }

        if let Some(ref stroke_color) = style.stroke_color {
            path = path.set("stroke", self.color_to_string(stroke_color));
            path = path.set("stroke-width", style.stroke_width);
            
            if style.stroke_opacity < 1.0 {
                path = path.set("stroke-opacity", style.stroke_opacity);
            }
            
            if !style.stroke_dash.is_empty() {
                let dash_array: Vec<String> = style.stroke_dash.iter()
                    .map(|&x| x.to_string())
                    .collect();
                path = path.set("stroke-dasharray", dash_array.join(","));
            }
            
            // Add line styling
            path = path.set("stroke-linecap", "round");
            path = path.set("stroke-linejoin", "round");
        }

        path
    }

    fn apply_style_to_circle(&self, mut circle: Circle, style: &ElementStyle) -> Circle {
        if let Some(ref fill_color) = style.fill_color {
            circle = circle.set("fill", self.color_to_string(fill_color));
            if style.fill_opacity < 1.0 {
                circle = circle.set("fill-opacity", style.fill_opacity);
            }
        }

        if let Some(ref stroke_color) = style.stroke_color {
            circle = circle.set("stroke", self.color_to_string(stroke_color));
            circle = circle.set("stroke-width", style.stroke_width);
            
            if style.stroke_opacity < 1.0 {
                circle = circle.set("stroke-opacity", style.stroke_opacity);
            }
        }

        circle
    }

    fn apply_style_to_text(&self, mut text: Text, style: &ElementStyle) -> Text {
        if let Some(ref fill_color) = style.fill_color {
            text = text.set("fill", self.color_to_string(fill_color));
        } else if let Some(ref stroke_color) = style.stroke_color {
            text = text.set("fill", self.color_to_string(stroke_color));
        }

        if let Some(ref font_family) = style.font_family {
            text = text.set("font-family", font_family.as_str());
        } else {
            text = text.set("font-family", "Arial, sans-serif");
        }

        text = text.set("font-size", style.font_size);
        
        if style.font_weight != 400 {
            text = text.set("font-weight", style.font_weight);
        }

        // Center text alignment
        text = text.set("text-anchor", "middle");
        text = text.set("dominant-baseline", "central");

        text
    }

    fn color_to_string(&self, color: &Color) -> String {
        if color.a == 255 {
            // Fully opaque, use RGB
            format!("rgb({},{},{})", color.r, color.g, color.b)
        } else {
            // Has transparency, use RGBA
            let alpha = color.a as f64 / 255.0;
            format!("rgba({},{},{},{})", color.r, color.g, color.b, alpha)
        }
    }

    fn round_coords(&self, coords: (f64, f64)) -> (f64, f64) {
        (
            self.round_value(coords.0),
            self.round_value(coords.1),
        )
    }

    fn round_value(&self, value: f64) -> f64 {
        let multiplier = 10_f64.powi(self.precision as i32);
        (value * multiplier).round() / multiplier
    }
}
