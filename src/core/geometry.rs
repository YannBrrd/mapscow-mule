use geo_types::{Coord, Point};
use nalgebra::Matrix3;

/// Geometry utilities for map processing
pub struct GeometryUtils;

impl GeometryUtils {
    /// Calculate the distance between two geographic points in meters
    pub fn haversine_distance(p1: &Point<f64>, p2: &Point<f64>) -> f64 {
        let lat1 = p1.y().to_radians();
        let lat2 = p2.y().to_radians();
        let delta_lat = (p2.y() - p1.y()).to_radians();
        let delta_lon = (p2.x() - p1.x()).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        6371000.0 * c // Earth's radius in meters
    }
    
    /// Simplify a line using the Douglas-Peucker algorithm
    pub fn simplify_line(points: &[Coord<f64>], tolerance: f64) -> Vec<Coord<f64>> {
        if points.len() <= 2 {
            return points.to_vec();
        }
        
        douglas_peucker(points, tolerance)
    }
    
    /// Calculate the area of a polygon in square meters
    pub fn polygon_area(coords: &[Coord<f64>]) -> f64 {
        if coords.len() < 3 {
            return 0.0;
        }
        
        let mut area = 0.0;
        let n = coords.len();
        
        for i in 0..n {
            let j = (i + 1) % n;
            area += coords[i].x * coords[j].y;
            area -= coords[j].x * coords[i].y;
        }
        
        area.abs() / 2.0
    }
    
    /// Check if a point is inside a polygon
    pub fn point_in_polygon(point: &Coord<f64>, polygon: &[Coord<f64>]) -> bool {
        let mut inside = false;
        let n = polygon.len();
        let mut j = n - 1;
        
        for i in 0..n {
            if ((polygon[i].y > point.y) != (polygon[j].y > point.y))
                && (point.x < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y) 
                    / (polygon[j].y - polygon[i].y) + polygon[i].x)
            {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
    
    /// Calculate the bounding box of a set of coordinates
    pub fn bounding_box(coords: &[Coord<f64>]) -> Option<(Coord<f64>, Coord<f64>)> {
        if coords.is_empty() {
            return None;
        }
        
        let mut min_x = coords[0].x;
        let mut max_x = coords[0].x;
        let mut min_y = coords[0].y;
        let mut max_y = coords[0].y;
        
        for coord in coords.iter().skip(1) {
            min_x = min_x.min(coord.x);
            max_x = max_x.max(coord.x);
            min_y = min_y.min(coord.y);
            max_y = max_y.max(coord.y);
        }
        
        Some((
            Coord { x: min_x, y: min_y },
            Coord { x: max_x, y: max_y },
        ))
    }
}

/// Douglas-Peucker line simplification algorithm
fn douglas_peucker(points: &[Coord<f64>], tolerance: f64) -> Vec<Coord<f64>> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    
    let mut max_distance = 0.0;
    let mut max_index = 0;
    let end = points.len() - 1;
    
    // Find the point with maximum distance from the line segment
    for i in 1..end {
        let distance = perpendicular_distance(&points[i], &points[0], &points[end]);
        if distance > max_distance {
            max_distance = distance;
            max_index = i;
        }
    }
    
    if max_distance > tolerance {
        // Recursively simplify both parts
        let mut result1 = douglas_peucker(&points[0..=max_index], tolerance);
        let result2 = douglas_peucker(&points[max_index..], tolerance);
        
        // Remove the duplicate point at the connection
        result1.pop();
        result1.extend(result2);
        result1
    } else {
        // Return only the endpoints
        vec![points[0], points[end]]
    }
}

/// Calculate perpendicular distance from a point to a line segment
fn perpendicular_distance(point: &Coord<f64>, line_start: &Coord<f64>, line_end: &Coord<f64>) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    
    if dx == 0.0 && dy == 0.0 {
        // Line segment is actually a point
        return ((point.x - line_start.x).powi(2) + (point.y - line_start.y).powi(2)).sqrt();
    }
    
    let numerator = ((line_end.y - line_start.y) * point.x 
                    - (line_end.x - line_start.x) * point.y 
                    + line_end.x * line_start.y 
                    - line_end.y * line_start.x).abs();
    let denominator = (dx.powi(2) + dy.powi(2)).sqrt();
    
    numerator / denominator
}

/// 2D transformation matrix for map projections and scaling
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    matrix: Matrix3<f64>,
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            matrix: Matrix3::identity(),
        }
    }
    
    pub fn translation(dx: f64, dy: f64) -> Self {
        let mut matrix = Matrix3::identity();
        matrix[(0, 2)] = dx;
        matrix[(1, 2)] = dy;
        Self { matrix }
    }
    
    pub fn scale(sx: f64, sy: f64) -> Self {
        let mut matrix = Matrix3::identity();
        matrix[(0, 0)] = sx;
        matrix[(1, 1)] = sy;
        Self { matrix }
    }
    
    pub fn rotation(angle_rad: f64) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        
        let mut matrix = Matrix3::identity();
        matrix[(0, 0)] = cos_a;
        matrix[(0, 1)] = -sin_a;
        matrix[(1, 0)] = sin_a;
        matrix[(1, 1)] = cos_a;
        
        Self { matrix }
    }
    
    pub fn compose(&self, other: &Transform2D) -> Self {
        Self {
            matrix: self.matrix * other.matrix,
        }
    }
    
    pub fn transform_point(&self, point: &Coord<f64>) -> Coord<f64> {
        let vec = nalgebra::Vector3::new(point.x, point.y, 1.0);
        let transformed = self.matrix * vec;
        
        Coord {
            x: transformed[0],
            y: transformed[1],
        }
    }
    
    pub fn transform_points(&self, points: &[Coord<f64>]) -> Vec<Coord<f64>> {
        points.iter().map(|p| self.transform_point(p)).collect()
    }
}
