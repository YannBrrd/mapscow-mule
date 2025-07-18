use std::path::Path;
use anyhow::Result;

// Import from the existing crate
use crate::parsers::{osm::OsmParser, Parser};

pub fn debug_roads_near_coordinates() -> Result<()> {
    println!("Searching for roads near specific coordinates...");
    
    // Load the OSM file
    let osm_file = "C:\\Users\\yann\\Documents\\Maperitive\\sartrouville\\sartrouville.osm";
    if !Path::new(osm_file).exists() {
        println!("Error: OSM file {} not found", osm_file);
        return Ok(());
    }
    
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    println!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Target coordinates
    let targets = vec![
        (48.9443224247288, 2.177457844649215, "Boulevard de Bezons target"),
        (48.94396813214317, 2.1806281043179876, "Rue Georges Bernanos target"),
    ];
    
    for (target_lat, target_lon, desc) in &targets {
        println!("\n=== Roads near {} ({:.6}, {:.6}) ===", desc, target_lat, target_lon);
        
        let mut nearby_roads = Vec::new();
        
        for way in map_data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                let mut min_distance = f64::INFINITY;
                let mut closest_coord = None;
                
                // Check all nodes in this way
                for &node_id in &way.nodes {
                    if let Some(node) = map_data.nodes.get(&node_id) {
                        let lat_diff = node.lat - target_lat;
                        let lon_diff = node.lon - target_lon;
                        let distance = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt();
                        
                        if distance < min_distance {
                            min_distance = distance;
                            closest_coord = Some((node.lat, node.lon));
                        }
                    }
                }
                
                // Include roads within 0.01 degrees (~1km)
                if min_distance <= 0.01 {
                    let name = way.tags.get("name").unwrap_or(&"<unnamed>".to_string()).clone();
                    nearby_roads.push((way.id, name, highway.clone(), min_distance, closest_coord));
                }
            }
        }
        
        // Sort by distance
        nearby_roads.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        
        println!("Found {} roads within 1km:", nearby_roads.len());
        for (i, (id, name, highway, distance, coord)) in nearby_roads.iter().take(15).enumerate() {
            println!("  {}: Way {} '{}' ({})", i+1, id, name, highway);
            println!("     Distance: {:.6} degrees (~{:.0}m)", distance, distance * 111000.0);
            if let Some((lat, lon)) = coord {
                println!("     Closest point: {:.6}, {:.6}", lat, lon);
            }
        }
        
        // Specifically look for streets with the name we're looking for
        let search_name = if desc.contains("Bezons") { "bezons" } else { "bernanos" };
        println!("\nSpecific search for '{}':", search_name);
        
        for (id, name, highway, distance, coord) in &nearby_roads {
            if name.to_lowercase().contains(search_name) {
                println!("  ✓ FOUND: Way {} '{}' ({})", id, name, highway);
                println!("    Distance: {:.6} degrees (~{:.0}m)", distance, distance * 111000.0);
                if let Some((lat, lon)) = coord {
                    println!("    Closest point: {:.6}, {:.6}", lat, lon);
                }
            }
        }
    }
    
    Ok(())
}
