use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Service for geocoding addresses using Nominatim API
pub struct GeocodingService {
    client: reqwest::Client,
    base_url: String,
}

/// Response from Nominatim API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NominatimResult {
    pub place_id: u64,
    pub licence: String,
    pub osm_type: String,
    pub osm_id: u64,
    pub lat: String,
    pub lon: String,
    pub display_name: String,
    pub r#type: String,
    pub importance: Option<f32>,
    pub icon: Option<String>,
}

impl GeocodingService {
    /// Create a new geocoding service
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://nominatim.openstreetmap.org".to_string(),
        }
    }

    /// Search for places matching the given query
    pub async fn search(&self, query: &str) -> Result<Vec<crate::gui::GeocodeResult>> {
        let url = format!("{}/search", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("limit", "10"),
                ("addressdetails", "1"),
                ("extratags", "1"),
                ("namedetails", "1"),
            ])
            .header("User-Agent", "mapscow-mule/0.1.0")
            .send()
            .await?;
        
        let nominatim_results: Vec<NominatimResult> = response.json().await?;
        
        let results = nominatim_results
            .into_iter()
            .filter_map(|result| {
                // Parse coordinates
                let lat = result.lat.parse::<f64>().ok()?;
                let lon = result.lon.parse::<f64>().ok()?;
                
                // Determine place type
                let place_type = match result.r#type.as_str() {
                    "city" | "town" | "village" => "Settlement",
                    "house" | "building" => "Building",
                    "road" | "street" => "Street",
                    "country" => "Country",
                    "state" => "State",
                    "county" => "County",
                    "postcode" => "Postal Code",
                    "amenity" => "Amenity",
                    "shop" => "Shop",
                    "leisure" => "Leisure",
                    "tourism" => "Tourism",
                    _ => "Other",
                }.to_string();
                
                Some(crate::gui::GeocodeResult {
                    display_name: result.display_name,
                    lat,
                    lon,
                    place_type,
                    importance: result.importance.unwrap_or(0.0),
                })
            })
            .collect();
        
        Ok(results)
    }

    /// Reverse geocode coordinates to get address
    pub async fn reverse_geocode(&self, lat: f64, lon: f64) -> Result<Option<String>> {
        let url = format!("{}/reverse", self.base_url);
        
        let lat_str = lat.to_string();
        let lon_str = lon.to_string();
        
        let response = self.client
            .get(&url)
            .query(&[
                ("lat", lat_str.as_str()),
                ("lon", lon_str.as_str()),
                ("format", "json"),
                ("addressdetails", "1"),
            ])
            .header("User-Agent", "mapscow-mule/0.1.0")
            .send()
            .await?;
        
        let result: NominatimResult = response.json().await?;
        Ok(Some(result.display_name))
    }
}

impl Default for GeocodingService {
    fn default() -> Self {
        Self::new()
    }
}
