use crate::core::{MapData, Node, Way, Relation, RelationMember, ElementType};
use crate::parsers::{Parser, ParseError};
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// OSM XML parser
pub struct OsmParser {
    /// Whether to include metadata (user, timestamp, etc.)
    include_metadata: bool,
}

impl OsmParser {
    pub fn new() -> Self {
        Self {
            include_metadata: false,
        }
    }
    
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

impl Parser<MapData> for OsmParser {
    fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<MapData> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);
        reader.config_mut().trim_text(true);
        
        self.parse_osm_xml(&mut reader)
    }
    
    fn parse_string(&self, content: &str) -> Result<MapData> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);
        
        self.parse_osm_xml(&mut reader)
    }
}

impl OsmParser {
    fn parse_osm_xml<R: std::io::BufRead>(&self, reader: &mut Reader<R>) -> Result<MapData> {
        let mut map_data = MapData::new();
        let mut buf = Vec::new();
        
        // Current element being parsed
        let mut current_element: Option<OsmElement> = None;
        let mut current_tags = HashMap::new();
        let mut current_way_nodes = Vec::new();
        let mut current_relation_members = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"osm" => {
                            // OSM root element - could parse version info here
                        }
                        b"node" => {
                            current_element = Some(self.parse_node_start(e)?);
                            current_tags.clear();
                        }
                        b"way" => {
                            current_element = Some(self.parse_way_start(e)?);
                            current_tags.clear();
                            current_way_nodes.clear();
                        }
                        b"relation" => {
                            current_element = Some(self.parse_relation_start(e)?);
                            current_tags.clear();
                            current_relation_members.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    match e.name().as_ref() {
                        b"tag" => {
                            let (key, value) = self.parse_tag(e)?;
                            current_tags.insert(key, value);
                        }
                        b"nd" => {
                            if let Some(node_ref) = self.parse_node_ref(e)? {
                                current_way_nodes.push(node_ref);
                            }
                        }
                        b"member" => {
                            if let Some(member) = self.parse_relation_member(e)? {
                                current_relation_members.push(member);
                            }
                        }
                        b"node" => {
                            // Self-closing node (no tags)
                            if let Some(element) = current_element.take() {
                                match element {
                                    OsmElement::Node { id, lat, lon } => {
                                        let node = Node {
                                            id,
                                            lat,
                                            lon,
                                            tags: HashMap::new(),
                                        };
                                        map_data.add_node(node);
                                    }
                                    _ => {}
                                }
                            } else {
                                // Parse self-closing node
                                let node_element = self.parse_node_start(e)?;
                                if let OsmElement::Node { id, lat, lon } = node_element {
                                    let node = Node {
                                        id,
                                        lat,
                                        lon,
                                        tags: HashMap::new(),
                                    };
                                    map_data.add_node(node);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"node" => {
                            if let Some(OsmElement::Node { id, lat, lon }) = current_element.take() {
                                let node = Node {
                                    id,
                                    lat,
                                    lon,
                                    tags: current_tags.clone(),
                                };
                                map_data.add_node(node);
                            }
                        }
                        b"way" => {
                            if let Some(OsmElement::Way { id }) = current_element.take() {
                                let is_closed = current_way_nodes.first() == current_way_nodes.last() 
                                    && current_way_nodes.len() > 2;
                                
                                let way = Way {
                                    id,
                                    nodes: current_way_nodes.clone(),
                                    tags: current_tags.clone(),
                                    is_closed,
                                };
                                map_data.add_way(way);
                            }
                        }
                        b"relation" => {
                            if let Some(OsmElement::Relation { id }) = current_element.take() {
                                let relation = Relation {
                                    id,
                                    members: current_relation_members.clone(),
                                    tags: current_tags.clone(),
                                };
                                map_data.add_relation(relation);
                            }
                        }
                        b"osm" => {
                            // End of document
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => {
                    break;
                }
                Err(e) => return Err(ParseError::Xml(format!("XML error at position {}: {}", 
                    reader.buffer_position(), e)).into()),
                _ => {
                    // Other events (Text, Comment, etc.)
                }
            }
            buf.clear();
        }
        
        Ok(map_data)
    }
    
    fn parse_node_start(&self, element: &quick_xml::events::BytesStart) -> Result<OsmElement> {
        let mut id = None;
        let mut lat = None;
        let mut lon = None;
        
        for attr in element.attributes() {
            let attr = attr?;
            match attr.key.as_ref() {
                b"id" => {
                    id = Some(std::str::from_utf8(&attr.value)?.parse::<i64>()?);
                }
                b"lat" => {
                    lat = Some(std::str::from_utf8(&attr.value)?.parse::<f64>()?);
                }
                b"lon" => {
                    lon = Some(std::str::from_utf8(&attr.value)?.parse::<f64>()?);
                }
                _ => {} // Skip other attributes for now
            }
        }
        
        let id = id.ok_or(ParseError::MissingField("id".to_string()))?;
        let lat = lat.ok_or(ParseError::MissingField("lat".to_string()))?;
        let lon = lon.ok_or(ParseError::MissingField("lon".to_string()))?;
        
        // Validate coordinates
        if lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0 {
            println!("Warning: Invalid coordinates found: lat={}, lon={}, skipping node {}", lat, lon, id);
            return Err(ParseError::InvalidCoordinate { lat, lon }.into());
        }
        
        // Additional sanity check for extreme outliers that might be data errors
        // For France/Europe, coordinates should be roughly:
        // Latitude: 40-60 degrees North, Longitude: -10 to 30 degrees East
        if lat < 30.0 || lat > 70.0 || lon < -20.0 || lon > 50.0 {
            println!("Warning: Suspicious coordinates (possible data error): lat={}, lon={}, node {}", lat, lon, id);
        }
        
        Ok(OsmElement::Node { id, lat, lon })
    }
    
    fn parse_way_start(&self, element: &quick_xml::events::BytesStart) -> Result<OsmElement> {
        for attr in element.attributes() {
            let attr = attr?;
            if attr.key.as_ref() == b"id" {
                let id = std::str::from_utf8(&attr.value)?.parse::<i64>()?;
                return Ok(OsmElement::Way { id });
            }
        }
        
        Err(ParseError::MissingField("id".to_string()).into())
    }
    
    fn parse_relation_start(&self, element: &quick_xml::events::BytesStart) -> Result<OsmElement> {
        for attr in element.attributes() {
            let attr = attr?;
            if attr.key.as_ref() == b"id" {
                let id = std::str::from_utf8(&attr.value)?.parse::<i64>()?;
                return Ok(OsmElement::Relation { id });
            }
        }
        
        Err(ParseError::MissingField("id".to_string()).into())
    }
    
    fn parse_tag(&self, element: &quick_xml::events::BytesStart) -> Result<(String, String)> {
        let mut key = None;
        let mut value = None;
        
        for attr in element.attributes() {
            let attr = attr?;
            match attr.key.as_ref() {
                b"k" => key = Some(std::str::from_utf8(&attr.value)?.to_string()),
                b"v" => value = Some(std::str::from_utf8(&attr.value)?.to_string()),
                _ => {}
            }
        }
        
        let key = key.ok_or(ParseError::MissingField("k".to_string()))?;
        let value = value.ok_or(ParseError::MissingField("v".to_string()))?;
        
        Ok((key, value))
    }
    
    fn parse_node_ref(&self, element: &quick_xml::events::BytesStart) -> Result<Option<i64>> {
        for attr in element.attributes() {
            let attr = attr?;
            if attr.key.as_ref() == b"ref" {
                let node_ref = std::str::from_utf8(&attr.value)?.parse::<i64>()?;
                return Ok(Some(node_ref));
            }
        }
        
        Ok(None)
    }
    
    fn parse_relation_member(&self, element: &quick_xml::events::BytesStart) -> Result<Option<RelationMember>> {
        let mut element_type = None;
        let mut id = None;
        let mut role = String::new();
        
        for attr in element.attributes() {
            let attr = attr?;
            match attr.key.as_ref() {
                b"type" => {
                    let type_str = std::str::from_utf8(&attr.value)?;
                    element_type = Some(match type_str {
                        "node" => ElementType::Node,
                        "way" => ElementType::Way,
                        "relation" => ElementType::Relation,
                        _ => return Err(ParseError::InvalidFormat(format!("Unknown element type: {}", type_str)).into()),
                    });
                }
                b"ref" => {
                    id = Some(std::str::from_utf8(&attr.value)?.parse::<i64>()?);
                }
                b"role" => {
                    role = std::str::from_utf8(&attr.value)?.to_string();
                }
                _ => {}
            }
        }
        
        if let (Some(element_type), Some(id)) = (element_type, id) {
            Ok(Some(RelationMember {
                element_type,
                id,
                role,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
enum OsmElement {
    Node { id: i64, lat: f64, lon: f64 },
    Way { id: i64 },
    Relation { id: i64 },
}

impl Default for OsmParser {
    fn default() -> Self {
        Self::new()
    }
}
