pub mod svg_export;
pub mod png_export;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::rendering::RenderedMap;

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
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub background_color: Option<crate::parsers::stylesheet::Color>,
    pub quality: Option<u8>, // For JPEG
    pub compression: Option<u8>, // For PNG
}

impl ExportOptions {
    pub fn new(format: ExportFormat, output_path: PathBuf, width: u32, height: u32) -> Self {
        Self {
            format,
            output_path,
            width,
            height,
            dpi: 300.0,
            background_color: None,
            quality: Some(90),
            compression: Some(6),
        }
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
    
    pub fn export(&self, rendered_map: &RenderedMap, format: ExportFormat, options: &ExportOptions) -> Result<()> {
        match format {
            ExportFormat::Svg => {
                svg_export::SvgExporter::new().export(rendered_map, options)
            }
            ExportFormat::Png => {
                png_export::PngExporter::new().export(rendered_map, options)
            }
            ExportFormat::Jpeg => {
                png_export::PngExporter::new().export_jpeg(rendered_map, options)
            }
            ExportFormat::Pdf => {
                // TODO: Implement PDF export
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
        
        if let Some(quality) = options.quality {
            if quality > 100 {
                return Err(anyhow::anyhow!("Quality must be between 0 and 100"));
            }
        }
        
        if let Some(compression) = options.compression {
            if compression > 9 {
                return Err(anyhow::anyhow!("Compression level must be between 0 and 9"));
            }
        }
        
        Ok(())
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new()
    }
}
