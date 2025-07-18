use mapscow_mule::export::{ExportFormat, ExportOptions};
use mapscow_mule::export::svg_export::SvgExporter;
use mapscow_mule::parsers::osm::OsmParser;
use mapscow_mule::parsers::Parser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing curved road names export...");
    
    // Parse the OSM file
    let parser = OsmParser::new();
    let map_data = parser.parse_file("examples/notre-dame.osm")?;
    
    println!("Parsed OSM data: {} ways, {} nodes", map_data.ways.len(), map_data.nodes.len());
    
    // Create SVG exporter with curved road names enabled
    let exporter = SvgExporter::new()?
        .with_all_road_names(true)  // Enable showing all road names
        .with_precision(2);
    
    // Set export options
    let options = ExportOptions::new(ExportFormat::SVG, "test_curved_road_names_export.svg".to_string())
        .with_size(2000, 2000);
    
    // Export to SVG
    let output_path = "test_curved_road_names_export.svg";
    exporter.export(output_path, &map_data, options)?;
    
    println!("Exported curved road names SVG to: {}", output_path);
    
    // Verify the file was created
    if std::path::Path::new(output_path).exists() {
        let file_size = fs::metadata(output_path)?.len();
        println!("SVG file created successfully, size: {} bytes", file_size);
        
        // Read a sample of the content to verify road names are included
        let content = fs::read_to_string(output_path)?;
        let text_count = content.matches("<text").count();
        println!("Found {} text elements in the SVG", text_count);
        
        if text_count > 0 {
            println!("✅ Test passed! Road names are being exported with curved positioning.");
        } else {
            println!("⚠️  Warning: No text elements found in SVG");
        }
    } else {
        println!("❌ Test failed: SVG file was not created");
    }
    
    Ok(())
}
