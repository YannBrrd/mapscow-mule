// Test file to verify Google Maps SVG export functionality
use mapscow_mule::export::svg_export::SvgExporter;
use mapscow_mule::export::{ExportOptions, ExportFormat};
use mapscow_mule::core::MapData;
use mapscow_mule::parsers::osm::OsmParser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    debug!("Testing Google Maps SVG export...");
    
    // Load the Notre Dame OSM data
    let osm_file = "examples/notre-dame.osm";
    debug!("Loading OSM file: {}", osm_file);
    
    let parser = OsmParser::new();
    let map_data = parser.parse_file(osm_file)?;
    
    debug!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Create SVG exporter with Google Maps styling
    let exporter = SvgExporter::new();
    
    // Export options
    let options = ExportOptions {
        width: 1200,
        height: 800,
        dpi: 300.0,
        background_color: crate::parsers::stylesheet::Color { r: 242, g: 239, b: 233, a: 255 }, // Google Maps background
        quality: 100,
    };
    
    // Export to SVG
    let output_file = "test_google_maps_output.svg";
    debug!("Exporting to: {}", output_file);
    
    exporter.export(
        &map_data,
        Path::new(output_file),
        &options,
        48.853, // Notre Dame latitude
        2.349,  // Notre Dame longitude
        1000.0, // Scale
    )?;
    
    debug!("✓ Google Maps style SVG export completed successfully!");
    debug!("Check {} for the result with Google Maps styling and Inkscape layers", output_file);
    
    Ok(())
}
