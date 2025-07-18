use mapscow_mule::{
    core::MapData,
    export::{ExportOptions, ExportFormat},
    export::svg_export::SvgExporter,
    rendering::engine::RenderingEngine,
    styles::StyleManager,
};
use anyhow::Result;

/// Example demonstrating improved SVG rendering capabilities
fn main() -> Result<()> {
    // This example shows how to use the improved SVG rendering
    println!("SVG Rendering Example");
    println!("=====================");
    
    // Create a sample map data (in a real application, this would come from OSM/GPX files)
    let map_data = create_sample_map_data();
    
    // Create improved SVG exporter with custom settings
    let svg_exporter = SvgExporter::new()
        .with_precision(2)           // Higher precision for smoother curves
        .with_anti_aliasing(true)    // Better visual quality
        .with_layer_separation(true); // Organized layer structure
    
    // Export using the direct data method (legacy support)
    println!("Exporting with direct data method...");
    svg_exporter.export_with_data(
        &map_data,
        "output/sample_map_direct.svg",
        1200,
        800,
        48.8566,  // Paris latitude
        2.3522,   // Paris longitude
        5000.0,   // Scale factor
    )?;
    
    // Example of using the advanced rendering pipeline
    println!("Exporting with advanced rendering pipeline...");
    
    // Create style manager and apply styles
    let style_manager = StyleManager::new();
    let styled_map = style_manager.apply_styles(&map_data)?;
    
    // Create advanced rendering engine
    let rendering_engine = RenderingEngine::new()
        .with_culling(true)                    // Enable viewport culling
        .with_simplification(true, 1.0);       // Enable line simplification
    
    // Create export options
    let export_options = ExportOptions {
        format: ExportFormat::Svg,
        output_path: "output/sample_map_advanced.svg".to_string(),
        width: 1200,
        height: 800,
        dpi: 300.0,
        background_color: None,
        quality: None,
        compression: None,
    };
    
    // Render with advanced features
    let rendered_map = rendering_engine.render_advanced(&styled_map, &export_options)?;
    
    // Export the rendered map
    svg_exporter.export(
        &rendered_map,
        &export_options.output_path,
        export_options.width,
        export_options.height,
    )?;
    
    println!("SVG files exported successfully!");
    println!("- output/sample_map_direct.svg (direct method)");
    println!("- output/sample_map_advanced.svg (advanced pipeline)");
    
    println!("\nFeatures demonstrated:");
    println!("- Improved coordinate precision and rounding");
    println!("- Layer-based organization (water, landuse, buildings, roads, POIs)");
    println!("- Enhanced styling with opacity and improved colors");
    println!("- Proper SVG structure with metadata");
    println!("- Line simplification for better performance");
    println!("- Viewport culling support");
    println!("- Anti-aliasing for smoother graphics");
    
    Ok(())
}

fn create_sample_map_data() -> MapData {
    use std::collections::HashMap;
    use mapscow_mule::core::{MapBounds, Node, Way};
    
    let mut nodes = HashMap::new();
    let mut ways = HashMap::new();
    
    // Create some sample nodes (simplified Paris area)
    let sample_nodes = vec![
        (1, 48.8566, 2.3522, "amenity", "restaurant"),  // Notre-Dame area
        (2, 48.8606, 2.3376, "amenity", "hospital"),    // Louvre area
        (3, 48.8584, 2.2945, "amenity", "school"),      // Eiffel Tower area
        (4, 48.8738, 2.2950, "landuse", "park"),        // Arc de Triomphe area
    ];
    
    for (id, lat, lon, key, value) in sample_nodes {
        let mut tags = HashMap::new();
        tags.insert(key.to_string(), value.to_string());
        if id == 1 {
            tags.insert("name".to_string(), "Sample Restaurant".to_string());
        }
        
        nodes.insert(id, Node {
            id,
            lat,
            lon,
            tags,
        });
    }
    
    // Create a sample way (simplified road)
    let mut road_tags = HashMap::new();
    road_tags.insert("highway".to_string(), "primary".to_string());
    road_tags.insert("name".to_string(), "Sample Street".to_string());
    
    ways.insert(1, Way {
        id: 1,
        nodes: vec![1, 2, 3, 4],
        tags: road_tags,
        is_closed: false,
    });
    
    // Create a sample building
    let mut building_tags = HashMap::new();
    building_tags.insert("building".to_string(), "yes".to_string());
    
    ways.insert(2, Way {
        id: 2,
        nodes: vec![1, 2, 3, 1], // Closed way
        tags: building_tags,
        is_closed: true,
    });
    
    MapData {
        bounds: MapBounds {
            min_lat: 48.85,
            max_lat: 48.88,
            min_lon: 2.29,
            max_lon: 2.36,
        },
        nodes,
        ways,
        relations: HashMap::new(),
        gpx_tracks: Vec::new(),
    }
}
