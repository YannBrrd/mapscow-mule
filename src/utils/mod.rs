pub mod file_dialog;
pub mod config;

use std::time::{SystemTime, UNIX_EPOCH};

/// Utility functions for the application
pub struct Utils;

impl Utils {
    /// Get current timestamp in milliseconds
    pub fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// Format a file size in bytes to human readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// Validate latitude value
    pub fn is_valid_latitude(lat: f64) -> bool {
        lat >= -90.0 && lat <= 90.0
    }
    
    /// Validate longitude value
    pub fn is_valid_longitude(lon: f64) -> bool {
        lon >= -180.0 && lon <= 180.0
    }
    
    /// Clamp a value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
    
    /// Linear interpolation between two values
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
    
    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }
    
    /// Convert radians to degrees
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }
}
