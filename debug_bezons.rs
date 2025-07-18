use mapscow_mule::parsers::{osm::OsmParser, Parser};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("Debugging: Searching for 'boulevard de bezons' in sartrouville.osm...");
    
    // Load the OSM file
    let osm_file = "C:\\Users\\yann\\Documents\\Maperitive\\sartrouville\\sartrouville.osm";
    if !Path::new(osm_file).exists() {
        println!("Error: OSM file {} not found", osm_file);
        return Ok(());
    }
    
    // Parse the OSM file
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    println!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Search for boulevard de bezons
    let mut found_roads = Vec::new();
    
    for way in map_data.ways.values() {
        if way.tags.contains_key("highway") {
            if let Some(name) = way.tags.get("name") {
                if name.to_lowercase().contains("bezons") || name.to_lowercase().contains("boulevard") {
                    found_roads.push((way.id, name.clone(), way.tags.get("highway").cloned()));
                }
            }
        }
    }
    
    println!("Found {} roads with 'bezons' or 'boulevard' in name:", found_roads.len());
    for (id, name, highway_type) in &found_roads {
        println!("  - Way ID: {}, Name: '{}', Highway: {:?}", id, name, highway_type);
    }
    
    // Search specifically for "boulevard de bezons"
    let mut specific_roads = Vec::new();
    
    for way in map_data.ways.values() {
        if way.tags.contains_key("highway") {
            if let Some(name) = way.tags.get("name") {
                if name.to_lowercase().contains("boulevard de bezons") {
                    specific_roads.push((way.id, name.clone(), way.tags.get("highway").cloned()));
                    
                    // Get coordinates of this road
                    let mut coords = Vec::new();
                    for &node_id in &way.nodes {
                        if let Some(node) = map_data.nodes.get(&node_id) {
                            coords.push((node.lat, node.lon));
                        }
                    }
                    
                    if !coords.is_empty() {
                        let first_coord = coords[0];
                        let last_coord = coords[coords.len() - 1];
                        println!("    First coordinate: {:.6}, {:.6}", first_coord.0, first_coord.1);
                        println!("    Last coordinate: {:.6}, {:.6}", last_coord.0, last_coord.1);
                        println!("    Total nodes: {}", coords.len());
                    }
                }
            }
        }
    }
    
    println!("Found {} roads specifically named 'boulevard de bezons':", specific_roads.len());
    for (id, name, highway_type) in &specific_roads {
        println!("  - Way ID: {}, Name: '{}', Highway: {:?}", id, name, highway_type);
    }
    
    if specific_roads.is_empty() {
        println!("No roads found with exact name 'boulevard de bezons'");
        println!("Let's check for variations...");
        
        for way in map_data.ways.values() {
            if way.tags.contains_key("highway") {
                if let Some(name) = way.tags.get("name") {
                    if name.to_lowercase().contains("bezons") {
                        println!("  - Found road with 'bezons': '{}' (highway: {:?})", 
                                name, way.tags.get("highway"));
                    }
                }
            }
        }
    }
    
    // Calculate the bounding box of all roads to understand data coverage
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    
    for way in map_data.ways.values() {
        if way.tags.contains_key("highway") {
            for &node_id in &way.nodes {
                if let Some(node) = map_data.nodes.get(&node_id) {
                    min_lat = min_lat.min(node.lat);
                    max_lat = max_lat.max(node.lat);
                    min_lon = min_lon.min(node.lon);
                    max_lon = max_lon.max(node.lon);
                }
            }
        }
    }
    
    println!("Road data bounding box:");
    println!("  Latitude: {:.6} to {:.6}", min_lat, max_lat);
    println!("  Longitude: {:.6} to {:.6}", min_lon, max_lon);
    
    Ok(())
}
