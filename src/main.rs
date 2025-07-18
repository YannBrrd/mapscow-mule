mod app;
mod core;
mod export;
mod gui;
mod parsers;
mod rendering;
mod styles;
mod utils;

use anyhow::Result;
use clap::{Arg, Command};
use env_logger;
use log::info;

use crate::app::MapscowMule;
use crate::parsers::{osm::OsmParser, Parser};
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "debug-bezons" {
        return debug_bezons_search();
    }
    
    if args.len() > 1 && args[1] == "debug-bernanos" {
        return debug_bernanos_rendering();
    }
    
    let matches = Command::new("mapscow-mule")
        .version("0.1.0")
        .author("Yann")
        .about("A Maperitive clone for high-quality SVG map generation")
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run in headless mode (no GUI)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path"),
        )
        .arg(
            Arg::new("osm")
                .long("osm")
                .value_name("FILE")
                .help("Load OSM file at startup")
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .get_matches();

    if matches.get_flag("headless") {
        info!("Starting in headless mode");
        // TODO: Implement headless mode for batch processing
        println!("Headless mode not implemented yet");
        return Ok(());
    }

    info!("Starting Mapscow Mule GUI");
    
    // Get optional OSM file from command line
    let osm_file = matches.get_one::<std::path::PathBuf>("osm").cloned();
    
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0]);
    
    // Add icon if available
    if let Some(icon) = load_app_icon() {
        viewport_builder = viewport_builder.with_icon(icon);
    }
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Mapscow Mule - Map Renderer",
        options,
        Box::new(move |_cc| Ok(Box::new(MapscowMule::new(osm_file)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))?;

    Ok(())
}

/// Load the application icon from embedded PNG data
fn load_app_icon() -> Option<egui::IconData> {
    // Check if the PNG file exists at compile time
    if !std::path::Path::new("assets/icons/mapscow-mule.png").exists() {
        println!("App icon not found. To add an icon:");
        println!("1. Convert assets/icons/mapscow-mule.svg to PNG (512x512)");
        println!("2. Save as assets/icons/mapscow-mule.png");
        println!("3. Rebuild the application");
        return None;
    }
    
    // Include the PNG file at compile time
    let icon_bytes = include_bytes!("../assets/icons/mapscow-mule.png");
    
    // Try to load the icon
    match eframe::icon_data::from_png_bytes(icon_bytes) {
        Ok(icon_data) => {
            info!("Successfully loaded application icon");
            Some(icon_data)
        }
        Err(e) => {
            eprintln!("Failed to load application icon: {}", e);
            None
        }
    }
}

fn debug_bezons_search() -> Result<()> {
    println!("Searching for roads near specific coordinates...");
    
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

fn debug_bernanos_rendering() -> Result<()> {
    println!("=== Debug: Rue Georges Bernanos Rendering ===");
    
    let osm_file = "C:\\Users\\yann\\Documents\\Maperitive\\sartrouville\\sartrouville.osm";
    if !Path::new(osm_file).exists() {
        println!("Error: OSM file {} not found", osm_file);
        return Ok(());
    }
    
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    println!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Find Rue Georges Bernanos specifically
    let mut bernanos_ways = Vec::new();
    for way in map_data.ways.values() {
        if let Some(name) = way.tags.get("name") {
            if name.to_lowercase().contains("bernanos") {
                bernanos_ways.push(way);
            }
        }
    }
    
    println!("Found {} ways with 'bernanos' in name:", bernanos_ways.len());
    
    // Also search for the specific way ID we know exists
    println!("\n--- Searching for Way ID 188677600 (known Rue Georges Bernanos) ---");
    if let Some(way) = map_data.ways.get(&188677600) {
        println!("✓ Found Way 188677600!");
        println!("  Name: {:?}", way.tags.get("name"));
        println!("  Highway: {:?}", way.tags.get("highway"));
        println!("  All tags: {:?}", way.tags);
    } else {
        println!("✗ Way 188677600 not found in parsed data");
    }
    
    // Check if any ways contain "Georges" or "Bernanos" separately
    let mut georges_ways = 0;
    let mut bernanos_ways_count = 0;
    for way in map_data.ways.values() {
        if let Some(name) = way.tags.get("name") {
            if name.to_lowercase().contains("georges") {
                georges_ways += 1;
                println!("Found way with 'georges': {}", name);
            }
            if name.to_lowercase().contains("bernanos") {
                bernanos_ways_count += 1;
                println!("Found way with 'bernanos': {}", name);
            }
        }
    }
    println!("Total ways with 'georges': {}", georges_ways);
    println!("Total ways with 'bernanos': {}", bernanos_ways_count);
    
    for way in &bernanos_ways {
        println!("\n--- Way {} ---", way.id);
        println!("Name: {:?}", way.tags.get("name"));
        println!("Highway: {:?}", way.tags.get("highway"));
        println!("All tags: {:?}", way.tags);
        println!("Node count: {}", way.nodes.len());
        
        // Get coordinates
        let mut coordinates = Vec::new();
        for &node_id in &way.nodes {
            if let Some(node) = map_data.nodes.get(&node_id) {
                coordinates.push((node.lat, node.lon));
                println!("  Node {}: {:.6}, {:.6}", node_id, node.lat, node.lon);
            }
        }
        
        if !coordinates.is_empty() {
            let first = coordinates[0];
            let last = coordinates[coordinates.len() - 1];
            
            // Calculate bounds
            let min_lat = coordinates.iter().map(|(lat, _)| lat).fold(f64::INFINITY, |a, &b| a.min(b));
            let max_lat = coordinates.iter().map(|(lat, _)| lat).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let min_lon = coordinates.iter().map(|(_, lon)| lon).fold(f64::INFINITY, |a, &b| a.min(b));
            let max_lon = coordinates.iter().map(|(_, lon)| lon).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            println!("  First point: {:.6}, {:.6}", first.0, first.1);
            println!("  Last point: {:.6}, {:.6}", last.0, last.1);
            println!("  Bounds: lat {:.6} to {:.6}, lon {:.6} to {:.6}", min_lat, max_lat, min_lon, max_lon);
            
            // Check distance to target coordinate
            let target = (48.94396813214317, 2.1806281043179876);
            let dist_to_first = ((first.0 - target.0).powi(2) + (first.1 - target.1).powi(2)).sqrt();
            let dist_to_last = ((last.0 - target.0).powi(2) + (last.1 - target.1).powi(2)).sqrt();
            
            println!("  Distance to target from first: {:.6} degrees (~{:.0}m)", 
                     dist_to_first, dist_to_first * 111000.0);
            println!("  Distance to target from last: {:.6} degrees (~{:.0}m)", 
                     dist_to_last, dist_to_last * 111000.0);
        }
    }
    
    // Check what's near the target coordinates
    let target = (48.94396813214317, 2.1806281043179876);
    println!("\n=== Roads within 100m of target ({:.6}, {:.6}) ===", target.0, target.1);
    
    let mut nearby_roads = Vec::new();
    for way in map_data.ways.values() {
        if let Some(highway) = way.tags.get("highway") {
            let mut min_distance = f64::INFINITY;
            
            for &node_id in &way.nodes {
                if let Some(node) = map_data.nodes.get(&node_id) {
                    let distance = ((node.lat - target.0).powi(2) + (node.lon - target.1).powi(2)).sqrt();
                    min_distance = min_distance.min(distance);
                }
            }
            
            // Within 100m (roughly 0.001 degrees)
            if min_distance <= 0.001 {
                let name = way.tags.get("name").unwrap_or(&"<unnamed>".to_string()).clone();
                nearby_roads.push((way.id, name, highway.clone(), min_distance));
            }
        }
    }
    
    nearby_roads.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    
    for (id, name, highway, distance) in nearby_roads.iter().take(10) {
        println!("  Way {}: '{}' ({}) - {:.6} degrees (~{:.0}m)", 
                 id, name, highway, distance, distance * 111000.0);
    }
    
    Ok(())
}
