use crate::export::ExportOptions;
use crate::rendering::{RenderedMap, RenderElement};
use crate::parsers::stylesheet::Color;
use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage, ImageFormat};
use imageproc::drawing::{draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_polygon_mut, draw_text_mut};
use imageproc::point::Point;
use rusttype::{Font, Scale};

/// PNG/JPEG raster exporter
pub struct PngExporter;

impl PngExporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn export(&self, rendered_map: &RenderedMap, options: &ExportOptions) -> Result<()> {
        let image = self.render_to_image(rendered_map, options)?;
        image.save_with_format(&options.output_path, ImageFormat::Png)?;
        Ok(())
    }
    
    pub fn export_jpeg(&self, rendered_map: &RenderedMap, options: &ExportOptions) -> Result<()> {
        let image = self.render_to_image(rendered_map, options)?;
        
        // Convert RGBA to RGB for JPEG
        let rgb_image = image::ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
            let pixel = image.get_pixel(x, y);
            image::Rgb([pixel[0], pixel[1], pixel[2]])
        });
        
        rgb_image.save_with_format(&options.output_path, ImageFormat::Jpeg)?;
        Ok(())
    }
    
    fn render_to_image(&self, rendered_map: &RenderedMap, options: &ExportOptions) -> Result<RgbaImage> {
        let mut image = ImageBuffer::new(options.width, options.height);
        
        // Set background color
        let bg_color = options.background_color
            .unwrap_or(Color::new(255, 255, 255, 255));
        let bg_rgba = Rgba([bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
        
        for pixel in image.pixels_mut() {
            *pixel = bg_rgba;
        }
        
        // Comment out font loading for now - skip text rendering in PNG export
        // let font = Font::try_from_bytes(include_bytes!("../../assets/fonts/NotoSans-Regular.ttf") as &[u8])
        //     .or_else(|| {
        //         // Fallback to a minimal embedded font or skip text rendering
        //         log::warn!("No font available for text rendering in PNG export");
        //         None
        //     });
        
        // Render elements in order
        for element in &rendered_map.elements {
            // Skip text rendering for now due to font issues
            // self.render_element(&mut image, element, &font)?;
        }
        
        Ok(image)
    }
    
    fn render_element(
        &self,
        image: &mut RgbaImage,
        element: &RenderElement,
        font: &Option<Font>,
    ) -> Result<()> {
        match element {
            RenderElement::Line { points, style } => {
                self.render_line(image, points, style)?;
            }
            RenderElement::Polygon { exterior, holes: _, style } => {
                // For now, we'll render polygons as filled shapes without holes
                self.render_polygon(image, exterior, style)?;
            }
            RenderElement::Circle { center, radius, style } => {
                self.render_circle(image, *center, *radius, style)?;
            }
            RenderElement::Text { position, text, style } => {
                self.render_text(image, *position, text, style, font)?;
            }
        }
        
        Ok(())
    }
    
    fn render_line(
        &self,
        image: &mut RgbaImage,
        points: &[(f64, f64)],
        style: &crate::rendering::ElementStyle,
    ) -> Result<()> {
        if points.len() < 2 {
            return Ok(());
        }
        
        let color = style.stroke_color.unwrap_or(Color::new(0, 0, 0, 255));
        let rgba = self.apply_opacity(color, style.stroke_opacity);
        
        for window in points.windows(2) {
            let start = (window[0].0 as f32, window[0].1 as f32);
            let end = (window[1].0 as f32, window[1].1 as f32);
            
            // Simple line drawing - in a production version, you'd want proper anti-aliasing
            // and line width support
            draw_antialiased_line_segment_mut(
                image,
                start,
                end,
                rgba,
                imageproc::pixelops::interpolate,
            );
        }
        
        Ok(())
    }
    
    fn render_polygon(
        &self,
        image: &mut RgbaImage,
        points: &[(f64, f64)],
        style: &crate::rendering::ElementStyle,
    ) -> Result<()> {
        if points.len() < 3 {
            return Ok(());
        }
        
        // Convert points to the format expected by imageproc
        let polygon_points: Vec<Point<i32>> = points
            .iter()
            .map(|(x, y)| Point::new(*x as i32, *y as i32))
            .collect();
        
        // Fill polygon if fill color is specified
        if let Some(fill_color) = style.fill_color {
            let rgba = self.apply_opacity(fill_color, style.fill_opacity);
            draw_polygon_mut(image, &polygon_points, rgba);
        }
        
        // Draw outline if stroke color is specified
        if let Some(stroke_color) = style.stroke_color {
            let rgba = self.apply_opacity(stroke_color, style.stroke_opacity);
            
            // Draw lines between consecutive points
            for i in 0..polygon_points.len() {
                let start = polygon_points[i];
                let end = polygon_points[(i + 1) % polygon_points.len()];
                
                draw_antialiased_line_segment_mut(
                    image,
                    (start.x as f32, start.y as f32),
                    (end.x as f32, end.y as f32),
                    rgba,
                    imageproc::pixelops::interpolate,
                );
            }
        }
        
        Ok(())
    }
    
    fn render_circle(
        &self,
        image: &mut RgbaImage,
        center: (f64, f64),
        radius: f64,
        style: &crate::rendering::ElementStyle,
    ) -> Result<()> {
        let center_point = Point::new(center.0 as i32, center.1 as i32);
        
        if let Some(fill_color) = style.fill_color {
            let rgba = self.apply_opacity(fill_color, style.fill_opacity);
            draw_filled_circle_mut(image, center_point, radius as i32, rgba);
        }
        
        // TODO: Add circle outline support
        
        Ok(())
    }
    
    fn render_text(
        &self,
        image: &mut RgbaImage,
        position: (f64, f64),
        text: &str,
        style: &crate::rendering::ElementStyle,
        font: &Option<Font>,
    ) -> Result<()> {
        if let Some(font) = font {
            let color = style.fill_color.unwrap_or(Color::new(0, 0, 0, 255));
            let rgba = self.apply_opacity(color, style.fill_opacity);
            
            let scale = Scale::uniform(style.font_size);
            
            draw_text_mut(
                image,
                rgba,
                position.0 as i32,
                position.1 as i32,
                scale,
                font,
                text,
            );
        }
        // If no font available, skip text rendering
        
        Ok(())
    }
    
    fn apply_opacity(&self, color: Color, opacity: f32) -> Rgba<u8> {
        let alpha = ((color.a as f32 * opacity) as u8).min(255);
        Rgba([color.r, color.g, color.b, alpha])
    }
}

impl Default for PngExporter {
    fn default() -> Self {
        Self::new()
    }
}
