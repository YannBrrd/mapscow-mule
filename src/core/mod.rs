pub mod geometry;
pub mod projection;

use geo_types::{Coord, LineString, Polygon};
use std::collections::HashMap;

/// Represents a complete map dataset with all geographic features
#[derive(Debug, Clone)]
pub struct MapData {
    pub bounds: MapBounds,
    pub nodes: HashMap<i64, Node>,
    pub ways: HashMap<i64, Way>,
    pub relations: HashMap<i64, Relation>,
    pub gpx_tracks: Vec<GpxTrack>,
}

/// Geographic bounds of the map area
#[derive(Debug, Clone, Copy)]
pub struct MapBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// OSM Node representing a point with coordinates and tags
#[derive(Debug, Clone)]
pub struct Node {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
    pub tags: HashMap<String, String>,
}

/// OSM Way representing a line or area with node references and tags
#[derive(Debug, Clone)]
pub struct Way {
    pub id: i64,
    pub nodes: Vec<i64>,
    pub tags: HashMap<String, String>,
    pub is_closed: bool,
}

/// OSM Relation representing a relationship between multiple elements
#[derive(Debug, Clone)]
pub struct Relation {
    pub id: i64,
    pub members: Vec<RelationMember>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RelationMember {
    pub element_type: ElementType,
    pub id: i64,
    pub role: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ElementType {
    Node,
    Way,
    Relation,
}

/// GPX track data
#[derive(Debug, Clone)]
pub struct GpxTrack {
    pub name: Option<String>,
    pub segments: Vec<GpxSegment>,
}

#[derive(Debug, Clone)]
pub struct GpxSegment {
    pub points: Vec<GpxPoint>,
}

#[derive(Debug, Clone)]
pub struct GpxPoint {
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f64>,
    pub time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Projection system for coordinate transformations
#[derive(Debug, Clone, Copy)]
pub enum ProjectionSystem {
    WebMercator,
    Utm { zone: u8, north: bool },
    LatLon,
}

impl MapData {
    pub fn new() -> Self {
        Self {
            bounds: MapBounds {
                min_lat: f64::INFINITY,
                max_lat: f64::NEG_INFINITY,
                min_lon: f64::INFINITY,
                max_lon: f64::NEG_INFINITY,
            },
            nodes: HashMap::new(),
            ways: HashMap::new(),
            relations: HashMap::new(),
            gpx_tracks: Vec::new(),
        }
    }
    
    pub fn add_node(&mut self, node: Node) {
        self.update_bounds(node.lat, node.lon);
        self.nodes.insert(node.id, node);
    }
    
    pub fn add_way(&mut self, way: Way) {
        self.ways.insert(way.id, way);
    }
    
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.insert(relation.id, relation);
    }
    
    pub fn add_gpx_track(&mut self, track: GpxTrack) {
        // Update bounds based on GPX track points
        for segment in &track.segments {
            for point in &segment.points {
                self.update_bounds(point.lat, point.lon);
            }
        }
        self.gpx_tracks.push(track);
    }
    
    fn update_bounds(&mut self, lat: f64, lon: f64) {
        self.bounds.min_lat = self.bounds.min_lat.min(lat);
        self.bounds.max_lat = self.bounds.max_lat.max(lat);
        self.bounds.min_lon = self.bounds.min_lon.min(lon);
        self.bounds.max_lon = self.bounds.max_lon.max(lon);
    }
    
    /// Get the geometry of a way as a LineString or Polygon
    pub fn get_way_geometry(&self, way: &Way) -> Option<geo_types::Geometry<f64>> {
        let coords: Vec<Coord<f64>> = way.nodes
            .iter()
            .filter_map(|&node_id| {
                self.nodes.get(&node_id).map(|node| Coord {
                    x: node.lon,
                    y: node.lat,
                })
            })
            .collect();
            
        if coords.is_empty() {
            return None;
        }
        
        if way.is_closed && coords.len() > 2 {
            // Create polygon
            let exterior = LineString::from(coords);
            Some(geo_types::Geometry::Polygon(Polygon::new(exterior, vec![])))
        } else {
            // Create linestring
            Some(geo_types::Geometry::LineString(LineString::from(coords)))
        }
    }
    
    /// Get all ways that match certain tag criteria
    pub fn get_ways_by_tags(&self, tag_filter: &HashMap<String, Vec<String>>) -> Vec<&Way> {
        self.ways
            .values()
            .filter(|way| {
                tag_filter.iter().any(|(key, values)| {
                    way.tags.get(key).map_or(false, |value| {
                        values.is_empty() || values.contains(value)
                    })
                })
            })
            .collect()
    }
}

impl MapBounds {
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
        )
    }
    
    pub fn width(&self) -> f64 {
        self.max_lon - self.min_lon
    }
    
    pub fn height(&self) -> f64 {
        self.max_lat - self.min_lat
    }
}

impl Default for MapData {
    fn default() -> Self {
        Self::new()
    }
}
