use mapscow_mule::{
    core::MapData,
    parsers::osm::OsmParser,
    export::svg_export::SvgExporter,
};
use anyhow::Result;

fn main() -> Result<()> {
    // Load the example OSM file
    let osm_path = "examples/notre-dame.osm";
    debug!("Loading OSM file: {}", osm_path);
    
    let mut parser = OsmParser::new();
    let map_data = parser.parse_file(osm_path)?;
    
    println!("Loaded {} nodes, {} ways", map_data.nodes.len(), map_data.ways.len());
    
    // Debug: check what roads we have
    for (id, way) in &map_data.ways {
        if let Some(highway) = way.tags.get("highway") {
            let name = way.tags.get("name").unwrap_or(&"<no name>".to_string());
            println!("Road {}: highway={}, name={}", id, highway, name);
        }
    }
    
    // Get map bounds to center the export
    let bounds = map_data.get_bounds();
    let center_lat = (bounds.min_lat + bounds.max_lat) / 2.0;
    let center_lon = (bounds.min_lon + bounds.max_lon) / 2.0;
    let scale = 100000.0; // Adjust scale as needed
    
    println!("Center: ({}, {}), Scale: {}", center_lat, center_lon, scale);
    
    // Export SVG with road names
    let exporter = SvgExporter::new()?;
    exporter.export_with_data(
        &map_data,
        "test_export_debug.svg",
        800,
        600,
        center_lat,
        center_lon,
        scale,
    )?;
    
    println!("Exported to test_export_debug.svg");
    Ok(())
}
