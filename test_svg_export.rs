use mapscow_mule::core::MapData;
use mapscow_mule::export::svg_export::SvgExporter;
use mapscow_mule::parsers::osm::OsmParser;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("Testing SVG export with road names...");
    
    // Load the example OSM file
    let osm_file = "examples/notre-dame.osm";
    if !Path::new(osm_file).exists() {
        println!("Error: OSM file {} not found", osm_file);
        return Ok(());
    }
    
    // Parse the OSM file
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    println!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Print some road information for debugging
    let mut road_count = 0;
    let mut named_road_count = 0;
    
    for way in map_data.ways.values() {
        if way.tags.contains_key("highway") {
            road_count += 1;
            if let Some(name) = way.tags.get("name") {
                named_road_count += 1;
                println!("Found road: {} (highway={})", name, way.tags.get("highway").unwrap_or("unknown"));
            }
        }
    }
    
    println!("Found {} roads total, {} with names", road_count, named_road_count);
    
    // Create the SVG exporter with enhanced settings
    let exporter = SvgExporter::new()
        .with_precision(3)
        .with_anti_aliasing(true)
        .with_layer_separation(true);
    
    // Calculate bounds for the map
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    
    for node in map_data.nodes.values() {
        min_lat = min_lat.min(node.lat);
        max_lat = max_lat.max(node.lat);
        min_lon = min_lon.min(node.lon);
        max_lon = max_lon.max(node.lon);
    }
    
    let center_lat = (min_lat + max_lat) / 2.0;
    let center_lon = (min_lon + max_lon) / 2.0;
    let scale = 100000.0; // Adjust this for zoom level
    
    println!("Map bounds: lat {:.6} to {:.6}, lon {:.6} to {:.6}", min_lat, max_lat, min_lon, max_lon);
    println!("Center: {:.6}, {:.6}", center_lat, center_lon);
    
    // Export to SVG with road names
    let output_path = "test_export_with_road_names.svg";
    exporter.export_with_data(
        &map_data,
        output_path,
        1200, // width
        800,  // height
        center_lat,
        center_lon,
        scale,
    )?;
    
    println!("SVG exported to: {}", output_path);
    println!("The export should now include road names for major roads!");
    
    Ok(())
}
