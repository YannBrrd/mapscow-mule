pub mod osm;
pub mod gpx;
pub mod stylesheet;

use anyhow::Result;
use std::path::Path;

/// Common trait for all parsers
pub trait Parser<T> {
    fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<T>;
    fn parse_string(&self, content: &str) -> Result<T>;
}

/// Error types for parsing operations
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("XML parsing error: {0}")]
    Xml(String),
    
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid coordinate: lat={lat}, lon={lon}")]
    InvalidCoordinate { lat: f64, lon: f64 },
}
