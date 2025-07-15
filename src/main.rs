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
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                // TODO: Add application icon
                eframe::icon_data::from_png_bytes(&[])
                    .unwrap_or_default()
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Mapscow Mule - Map Renderer",
        options,
        Box::new(|_cc| Ok(Box::new(MapscowMule::new()))),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))?;

    Ok(())
}
