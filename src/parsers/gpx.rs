use crate::core::{GpxTrack, GpxSegment, GpxPoint};
use crate::parsers::{Parser, ParseError};
use anyhow::Result;
use std::path::Path;

/// GPX file parser
pub struct GpxParser;

impl GpxParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser<Vec<GpxTrack>> for GpxParser {
    fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<GpxTrack>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_string(&content)
    }
    
    fn parse_string(&self, content: &str) -> Result<Vec<GpxTrack>> {
        // Try to use the gpx crate first
        match gpx::read(content.as_bytes()) {
            Ok(gpx_data) => {
                let mut tracks = Vec::new();
                
                for track in gpx_data.tracks {
                    let mut segments = Vec::new();
                    
                    for segment in track.segments {
                        let points: Vec<GpxPoint> = segment.points
                            .into_iter()
                            .map(|waypoint| GpxPoint {                    lat: waypoint.point().y(),
                    lon: waypoint.point().x(),
                    elevation: waypoint.elevation,
                    time: waypoint.time.map(|_t| chrono::Utc::now()), // Placeholder - use current time
                            })
                            .collect();
                        
                        segments.push(GpxSegment { points });
                    }
                    
                    tracks.push(GpxTrack {
                        name: track.name,
                        segments,
                    });
                }
                
                Ok(tracks)
            }
            Err(e) => {
                // Fallback to manual parsing if the gpx crate fails
                log::warn!("GPX crate failed, falling back to manual parsing: {}", e);
                self.parse_gpx_manual(content)
            }
        }
    }
}

impl GpxParser {
    /// Manual GPX parsing as fallback
    fn parse_gpx_manual(&self, content: &str) -> Result<Vec<GpxTrack>> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        
        let mut tracks = Vec::new();
        let mut current_track: Option<GpxTrack> = None;
        let mut current_segment: Option<GpxSegment> = None;
        let mut current_point: Option<GpxPoint> = None;
        let mut current_text = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"trk" => {
                            current_track = Some(GpxTrack {
                                name: None,
                                segments: Vec::new(),
                            });
                        }
                        b"trkseg" => {
                            current_segment = Some(GpxSegment {
                                points: Vec::new(),
                            });
                        }
                        b"trkpt" => {
                            let mut lat = None;
                            let mut lon = None;
                            
                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"lat" => {
                                        lat = Some(std::str::from_utf8(&attr.value)?.parse::<f64>()?);
                                    }
                                    b"lon" => {
                                        lon = Some(std::str::from_utf8(&attr.value)?.parse::<f64>()?);
                                    }
                                    _ => {}
                                }
                            }
                            
                            if let (Some(lat), Some(lon)) = (lat, lon) {
                                // Validate coordinates
                                if lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0 {
                                    return Err(ParseError::InvalidCoordinate { lat, lon }.into());
                                }
                                
                                current_point = Some(GpxPoint {
                                    lat,
                                    lon,
                                    elevation: None,
                                    time: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"name" => {
                            if let Some(ref mut track) = current_track {
                                track.name = Some(current_text.trim().to_string());
                            }
                        }
                        b"ele" => {
                            if let Some(ref mut point) = current_point {
                                if let Ok(elevation) = current_text.trim().parse::<f64>() {
                                    point.elevation = Some(elevation);
                                }
                            }
                        }
                        b"time" => {
                            if let Some(ref mut point) = current_point {
                                // Try to parse ISO 8601 timestamp
                                if let Ok(time) = chrono::DateTime::parse_from_rfc3339(current_text.trim()) {
                                    point.time = Some(time.with_timezone(&chrono::Utc));
                                }
                            }
                        }
                        b"trkpt" => {
                            if let (Some(point), Some(ref mut segment)) = (current_point.take(), &mut current_segment) {
                                segment.points.push(point);
                            }
                        }
                        b"trkseg" => {
                            if let (Some(segment), Some(ref mut track)) = (current_segment.take(), &mut current_track) {
                                track.segments.push(segment);
                            }
                        }
                        b"trk" => {
                            if let Some(track) = current_track.take() {
                                tracks.push(track);
                            }
                        }
                        _ => {}
                    }
                    current_text.clear();
                }
                Ok(Event::Text(e)) => {
                    current_text.push_str(&e.unescape()?);
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::Xml(format!("XML error at position {}: {}", 
                        reader.buffer_position(), e)).into());
                }
                _ => {}
            }
            buf.clear();
        }
        
        Ok(tracks)
    }
}

impl Default for GpxParser {
    fn default() -> Self {
        Self::new()
    }
}
