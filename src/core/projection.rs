use crate::core::ProjectionSystem;
use geo_types::Coord;

/// Coordinate projection utilities
pub struct Projector {
    system: ProjectionSystem,
}

impl Projector {
    pub fn new(system: ProjectionSystem) -> Self {
        Self { system }
    }
    
    /// Project geographic coordinates (lat, lon) to map coordinates (x, y)
    pub fn project(&self, lat: f64, lon: f64) -> Coord<f64> {
        match self.system {
            ProjectionSystem::WebMercator => self.to_web_mercator(lat, lon),
            ProjectionSystem::Utm { zone, north } => self.to_utm(lat, lon, zone, north),
            ProjectionSystem::LatLon => Coord { x: lon, y: lat },
        }
    }
    
    /// Inverse projection from map coordinates to geographic coordinates
    pub fn unproject(&self, x: f64, y: f64) -> (f64, f64) {
        match self.system {
            ProjectionSystem::WebMercator => self.from_web_mercator(x, y),
            ProjectionSystem::Utm { zone, north } => self.from_utm(x, y, zone, north),
            ProjectionSystem::LatLon => (y, x),
        }
    }
    
    /// Web Mercator projection (EPSG:3857)
    fn to_web_mercator(&self, lat: f64, lon: f64) -> Coord<f64> {
        const EARTH_RADIUS: f64 = 6378137.0;
        
        let x = EARTH_RADIUS * lon.to_radians();
        let y = EARTH_RADIUS * (std::f64::consts::PI / 4.0 + lat.to_radians() / 2.0).tan().ln();
        
        Coord { x, y }
    }
    
    /// Inverse Web Mercator projection
    fn from_web_mercator(&self, x: f64, y: f64) -> (f64, f64) {
        const EARTH_RADIUS: f64 = 6378137.0;
        
        let lon = (x / EARTH_RADIUS).to_degrees();
        let lat = (2.0 * (y / EARTH_RADIUS).exp().atan() - std::f64::consts::PI / 2.0).to_degrees();
        
        (lat, lon)
    }
    
    /// UTM projection (simplified version)
    fn to_utm(&self, lat: f64, lon: f64, zone: u8, north: bool) -> Coord<f64> {
        // Simplified UTM projection - in a real implementation, you'd use a proper library like proj
        let central_meridian = (zone as f64 - 1.0) * 6.0 - 180.0 + 3.0;
        let lon_diff = lon - central_meridian;
        
        let lat_rad = lat.to_radians();
        let lon_diff_rad = lon_diff.to_radians();
        
        // Simplified calculation - this is not accurate for production use
        let k0 = 0.9996; // Scale factor
        let a = 6378137.0; // Semi-major axis
        let e2 = 0.00669438; // First eccentricity squared
        
        let n = a / (1.0 - e2 * lat_rad.sin().powi(2)).sqrt();
        let t = lat_rad.tan();
        let c = e2 * lat_rad.cos().powi(2) / (1.0 - e2);
        let a_val = lat_rad.cos() * lon_diff_rad;
        
        let x = k0 * n * (a_val + (1.0 - t.powi(2) + c) * a_val.powi(3) / 6.0) + 500000.0;
        let mut y = k0 * (self.meridional_arc(lat_rad) + n * t * (a_val.powi(2) / 2.0));
        
        if !north {
            y += 10000000.0; // False northing for southern hemisphere
        }
        
        Coord { x, y }
    }
    
    /// Inverse UTM projection (simplified)
    fn from_utm(&self, x: f64, y: f64, zone: u8, north: bool) -> (f64, f64) {
        // Simplified inverse UTM - this is a placeholder
        // In practice, you'd use a proper geodetic library
        let central_meridian = (zone as f64 - 1.0) * 6.0 - 180.0 + 3.0;
        
        // Very rough approximation
        let lon = central_meridian + (x - 500000.0) / 111320.0;
        let mut y_adj = y;
        
        if !north {
            y_adj -= 10000000.0;
        }
        
        let lat = y_adj / 111320.0;
        
        (lat, lon)
    }
    
    /// Calculate meridional arc (simplified)
    fn meridional_arc(&self, lat: f64) -> f64 {
        const A: f64 = 6378137.0;
        const E2: f64 = 0.00669438;
        
        let e4 = E2 * E2;
        let e6 = e4 * E2;
        
        let m = A * ((1.0 - E2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0) * lat
            - (3.0 * E2 / 8.0 + 3.0 * e4 / 32.0 + 45.0 * e6 / 1024.0) * (2.0 * lat).sin()
            + (15.0 * e4 / 256.0 + 45.0 * e6 / 1024.0) * (4.0 * lat).sin()
            - (35.0 * e6 / 3072.0) * (6.0 * lat).sin());
        
        m
    }
}

/// Utility functions for coordinate system conversions
pub struct CoordinateUtils;

impl CoordinateUtils {
    /// Determine the UTM zone for a given longitude
    pub fn utm_zone_from_lon(lon: f64) -> u8 {
        ((lon + 180.0) / 6.0).floor() as u8 + 1
    }
    
    /// Determine if coordinates are in the northern hemisphere
    pub fn is_northern_hemisphere(lat: f64) -> bool {
        lat >= 0.0
    }
    
    /// Calculate the scale factor for a given latitude in Web Mercator
    pub fn web_mercator_scale_factor(lat: f64) -> f64 {
        1.0 / lat.to_radians().cos()
    }
    
    /// Convert degrees to decimal degrees (for DMS input)
    pub fn dms_to_decimal(degrees: i32, minutes: i32, seconds: f64) -> f64 {
        degrees.abs() as f64 + minutes as f64 / 60.0 + seconds / 3600.0
    }
    
    /// Convert decimal degrees to DMS
    pub fn decimal_to_dms(decimal: f64) -> (i32, i32, f64) {
        let abs_decimal = decimal.abs();
        let degrees = abs_decimal.floor() as i32;
        let minutes_float = (abs_decimal - degrees as f64) * 60.0;
        let minutes = minutes_float.floor() as i32;
        let seconds = (minutes_float - minutes as f64) * 60.0;
        
        let sign = if decimal < 0.0 { -1 } else { 1 };
        (sign * degrees, minutes, seconds)
    }
}
