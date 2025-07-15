use svg::node::element::{Group, Rectangle, Text, Path, Circle};
use svg::node::element::path::Data;
use svg::Document;
use anyhow::Result;
use crate::rendering::RenderedMap;
use crate::core::MapData;
use std::collections::HashMap;

pub struct SvgExporter {
    pub precision: usize,
}

impl SvgExporter {
    pub fn new() -> Self {
        Self {
            precision: 3,
        }
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
            .set("height", height);

        // Create main group for all elements
        let mut main_group = Group::new().set("id", "map");

        // Draw background
        let background = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("fill", "#f8f6f0"); // Google Maps-style beige background

        main_group = main_group.add(background);

        // Convert geographic coordinates to SVG coordinates
        let to_svg_coords = |lat: f64, lon: f64| -> (f64, f64) {
            let x = (width as f64 / 2.0) + (lon - center_lon) * scale;
            let y = (height as f64 / 2.0) - (lat - center_lat) * scale; // Flip Y for SVG
            (x, y)
        };

        // Draw water bodies first (lowest layer)
        for way in map_data.ways.values() {
            if let Some(natural) = way.tags.get("natural") {
                if natural == "water" || way.tags.contains_key("waterway") {
                    if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                        let water_path = Path::new()
                            .set("d", path_data)
                            .set("fill", "#add8e6") // Light blue for water
                            .set("stroke", "#6bb6ff")
                            .set("stroke-width", 1);
                        main_group = main_group.add(water_path);
                    }
                }
            }
        }

        // Draw land use areas (parks, forests)
        for way in map_data.ways.values() {
            if let Some(landuse) = way.tags.get("landuse") {
                let (fill_color, stroke_color) = match landuse.as_str() {
                    "forest" | "wood" => ("#90ee90", "#228b22"), // Light green, forest green
                    "grass" | "meadow" => ("#9acd32", "#6b8e23"), // Yellow green, olive
                    "residential" => ("#f0f0f0", "#d0d0d0"), // Light gray
                    "commercial" => ("#ffefd5", "#deb887"), // Papaya whip, burlywood
                    _ => ("#e0e0e0", "#c0c0c0"), // Default gray
                };
                
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let area_path = Path::new()
                        .set("d", path_data)
                        .set("fill", fill_color)
                        .set("stroke", stroke_color)
                        .set("stroke-width", 1);
                    main_group = main_group.add(area_path);
                }
            }
        }

        // Draw buildings
        for way in map_data.ways.values() {
            if way.tags.contains_key("building") {
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let building_path = Path::new()
                        .set("d", path_data)
                        .set("fill", "#d3d3d3") // Light gray for buildings
                        .set("stroke", "#808080") // Gray outline
                        .set("stroke-width", 1);
                    main_group = main_group.add(building_path);
                }
            }
        }

        // Draw roads
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                let (stroke_color, stroke_width) = match highway.as_str() {
                    "motorway" | "trunk" => ("#ff6600", 4),
                    "primary" => ("#ff9900", 3),
                    "secondary" => ("#ffcc00", 2),
                    "tertiary" | "residential" => ("#ffffff", 2),
                    "service" | "track" => ("#f0f0f0", 1),
                    _ => ("#e0e0e0", 1),
                };
                
                if let Some(path_data) = self.way_to_svg_path(way, map_data, &to_svg_coords) {
                    let road_path = Path::new()
                        .set("d", path_data)
                        .set("fill", "none")
                        .set("stroke", stroke_color)
                        .set("stroke-width", stroke_width)
                        .set("stroke-linecap", "round")
                        .set("stroke-linejoin", "round");
                    main_group = main_group.add(road_path);
                }
            }
        }

        // Draw POIs (points of interest)
        for node in map_data.nodes.values() {
            if let Some(amenity) = node.tags.get("amenity") {
                let (x, y) = to_svg_coords(node.lat, node.lon);
                if x >= 0.0 && x <= width as f64 && y >= 0.0 && y <= height as f64 {
                    let color = match amenity.as_str() {
                        "restaurant" | "cafe" => "#ff4444",
                        "hospital" => "#ff0000",
                        "school" => "#4444ff",
                        "bank" => "#00aa00",
                        _ => "#888888",
                    };
                    
                    let poi_circle = Circle::new()
                        .set("cx", x)
                        .set("cy", y)
                        .set("r", 3)
                        .set("fill", color)
                        .set("stroke", "#ffffff")
                        .set("stroke-width", 1);
                    main_group = main_group.add(poi_circle);
                }
            }
        }

        // Add a title
        let title = Text::new("Generated with Mapscow Mule")
            .set("x", width / 2)
            .set("y", 20)
            .set("text-anchor", "middle")
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 14)
            .set("fill", "#333333");

        main_group = main_group.add(title);
        document = document.add(main_group);

        // Write to file
        svg::save(output_path, &document)?;
        Ok(())
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

        for node_id in &way.nodes {
            if let Some(node) = map_data.nodes.get(node_id) {
                let (x, y) = to_svg_coords(node.lat, node.lon);
                
                if first {
                    data = data.move_to((x, y));
                    first = false;
                } else {
                    data = data.line_to((x, y));
                }
            }
        }

        // Close path if it's a closed way (area)
        if way.is_closed && way.nodes.len() > 2 {
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
            .set("height", height);

        // Create main group for all elements
        let mut main_group = Group::new().set("id", "map");

        // For now, create a simple placeholder
        let placeholder = Rectangle::new()
            .set("x", 10)
            .set("y", 10)
            .set("width", width - 20)
            .set("height", height - 20)
            .set("fill", "lightblue")
            .set("stroke", "blue")
            .set("stroke-width", 2);

        // Add a title
        let title = Text::new("Generated with Mapscow Mule")
            .set("x", width / 2)
            .set("y", 30)
            .set("text-anchor", "middle")
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 16);

        main_group = main_group.add(placeholder);
        main_group = main_group.add(title);

        document = document.add(main_group);

        // Write to file
        std::fs::write(output_path, document.to_string())?;
        Ok(())
    }
}
