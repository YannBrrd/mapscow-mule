// Test script to verify SVG export functionality
// This will help us verify if the road name export is working correctly

use std::path::Path;
use mapscow_mule::export::{ExportFormat, ExportOptions};
use mapscow_mule::parsers::osm::OsmParser;
use mapscow_mule::parsers::Parser;
use mapscow_mule::export::svg_export::SvgExporter;
use mapscow_mule::rendering::MapRenderer;
use mapscow_mule::export::Exporter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing SVG export with road names debug...");
    
    // Load the small OSM test file
    let osm_file = Path::new("examples/notre-dame.osm");
    
    if !osm_file.exists() {
        println!("❌ OSM file not found: {}", osm_file.display());
        return Ok(());
    }
    
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    println!("✅ Loaded OSM data:");
    println!("  - {} nodes", map_data.nodes.len());
    println!("  - {} ways", map_data.ways.len());
    
    // Find roads with names
    let mut roads_with_names = Vec::new();
    for (id, way) in &map_data.ways {
        if way.tags.contains_key("highway") {
            let highway = way.tags.get("highway").unwrap();
            let unnamed = "(unnamed)".to_string();
            let name = way.tags.get("name").unwrap_or(&unnamed);
            roads_with_names.push((*id, highway.clone(), name.clone()));
        }
    }
    
    println!("✅ Found {} roads:", roads_with_names.len());
    for (id, highway, name) in &roads_with_names {
        println!("  - Way {}: {} ({})", id, name, highway);
    }
    
    // Test export with show_all_road_names=true
    let output_path = "test_export_with_debug.svg";
    let options = ExportOptions::new(ExportFormat::Svg, output_path.to_string());
    let renderer = MapRenderer::new();
    
    let exporter = Exporter::new();
    
    // Use Notre Dame coordinates for center
    let center_lat = 48.8530;
    let center_lon = 2.3499;
    let scale = 10000.0;
    
    println!("📝 Exporting with show_all_road_names=true...");
    println!("   Center: ({}, {})", center_lat, center_lon);
    println!("   Scale: {}", scale);
    println!("   Output: {}", output_path);
    
    match exporter.export_map_with_viewport(
        &map_data,
        &renderer,
        &options,
        center_lat,
        center_lon,
        scale,
        true, // show_all_road_names=true
    ) {
        Ok(_) => {
            println!("✅ Export completed successfully!");
            
            // Check if the file was created
            if Path::new(output_path).exists() {
                println!("✅ SVG file created: {}", output_path);
                
                // Try to read and check the contents
                let svg_content = std::fs::read_to_string(output_path)?;
                let contains_road_name = svg_content.contains("Quai de l'Archevêché");
                
                println!("📄 SVG file size: {} bytes", svg_content.len());
                println!("🔍 Contains road name 'Quai de l'Archevêché': {}", contains_road_name);
                
                if contains_road_name {
                    println!("🎉 SUCCESS: Road names are being exported correctly!");
                } else {
                    println!("⚠️  Warning: Road name not found in SVG content");
                    println!("   The export may have worked but the road might not be visible in the current viewport");
                }
            } else {
                println!("❌ SVG file was not created");
            }
        }
        Err(e) => {
            println!("❌ Export failed: {}", e);
        }
    }
    
    Ok(())
}
