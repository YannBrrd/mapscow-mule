pub mod svg_export;
// pub mod png_export; // Disabled for now due to compatibility issues

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::rendering::RenderedMap;
use crate::core::MapData;
use crate::rendering::MapRenderer;

/// Available export formats
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Svg,
    Png,
    Jpeg,
    Pdf,
}

/// Export configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub background_color: Option<crate::parsers::stylesheet::Color>,
    pub quality: Option<u8>, // For JPEG
    pub compression: Option<u8>, // For PNG
}

impl ExportOptions {
    pub fn new(format: ExportFormat, output_path: String) -> Self {
        Self {
            format,
            output_path,
            width: 1024,
            height: 768,
            dpi: 300.0,
            background_color: None,
            quality: Some(90),
            compression: Some(6),
        }
    }
    
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    pub fn with_dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }
    
    pub fn with_background(mut self, color: crate::parsers::stylesheet::Color) -> Self {
        self.background_color = Some(color);
        self
    }
    
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality);
        self
    }
}

/// Main exporter that handles different output formats
pub struct Exporter;

impl Exporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn export_map_with_viewport(
        &self,
        map_data: &MapData,
        _renderer: &MapRenderer,
        options: &ExportOptions,
        center_lat: f64,
        center_lon: f64,
        scale: f64,
        show_all_road_names: bool,
    ) -> Result<()> {
        match options.format {
            ExportFormat::Svg => {
                let exporter = svg_export::SvgExporter::new()?
                    .with_all_road_names(show_all_road_names);
                exporter.export_with_data(
                    map_data,
                    &options.output_path, 
                    options.width, 
                    options.height,
                    center_lat,
                    center_lon,
                    scale,
                )
            }
            ExportFormat::Png => {
                Err(anyhow::anyhow!("PNG export not available yet - use SVG instead"))
            }
            ExportFormat::Jpeg => {
                Err(anyhow::anyhow!("JPEG export not available yet - use SVG instead"))
            }
            ExportFormat::Pdf => {
                Err(anyhow::anyhow!("PDF export not implemented yet"))
            }
        }
    }

    pub fn export_map(
        &self,
        _map_data: &MapData,
        _renderer: &MapRenderer,
        options: &ExportOptions,
    ) -> Result<()> {
        // Create a minimal rendered map for now
        let rendered_map = RenderedMap {
            elements: Vec::new(),
        };
        
        match options.format {
            ExportFormat::Svg => {
                svg_export::SvgExporter::new()?.export(
                    &rendered_map, 
                    &options.output_path, 
                    options.width, 
                    options.height
                )
            }
            ExportFormat::Png => {
                Err(anyhow::anyhow!("PNG export not available yet - use SVG instead"))
            }
            ExportFormat::Jpeg => {
                Err(anyhow::anyhow!("JPEG export not available yet - use SVG instead"))
            }
            ExportFormat::Pdf => {
                Err(anyhow::anyhow!("PDF export not implemented yet"))
            }
        }
    }
    
    /// Get the appropriate file extension for a format
    pub fn get_extension(format: ExportFormat) -> &'static str {
        match format {
            ExportFormat::Svg => "svg",
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Pdf => "pdf",
        }
    }
    
    /// Validate export options
    pub fn validate_options(&self, options: &ExportOptions) -> Result<()> {
        if options.width == 0 || options.height == 0 {
            return Err(anyhow::anyhow!("Width and height must be greater than 0"));
        }
        
        if options.dpi <= 0.0 {
            return Err(anyhow::anyhow!("DPI must be greater than 0"));
        }
        
        // Check if output directory exists
        if let Some(parent) = Path::new(&options.output_path).parent() {
            if !parent.exists() {
                return Err(anyhow::anyhow!("Output directory does not exist: {}", parent.display()));
            }
        }
        
        Ok(())
    }
}
