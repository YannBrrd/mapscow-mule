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

fn main() -> Result<()> {
    env_logger::init();
    
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
        .get_matches();

    if matches.get_flag("headless") {
        info!("Starting in headless mode");
        // TODO: Implement headless mode for batch processing
        println!("Headless mode not implemented yet");
        return Ok(());
    }

    info!("Starting Mapscow Mule GUI");
    
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
        Box::new(|_cc| Ok(Box::new(MapscowMule::new()))),
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
