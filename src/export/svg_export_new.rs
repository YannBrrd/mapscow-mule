use svg::node::element::{Circle, Group, Path, Rectangle, Text};
use svg::node::Text as TextNode;
use svg::Document;
use anyhow::Result;
use crate::rendering::RenderedMap;

pub struct SvgExporter {
    pub precision: usize,
}

impl SvgExporter {
    pub fn new() -> Self {
        Self {
            precision: 3,
        }
    }

    pub fn export<P: AsRef<std::path::Path>>(
        &self,
        rendered_map: &RenderedMap,
        output_path: P,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let mut document = Document::new()
            .set("viewBox", (0, 0, width, height))
            .set("width", width)
            .set("height", height);

        // Create main group for all elements
        let mut main_group = Group::new().set("id", "map");

        // For now, create a simple placeholder
        let placeholder = Rectangle::new()
            .set("x", 10)
            .set("y", 10)
            .set("width", width - 20)
            .set("height", height - 20)
            .set("fill", "lightblue")
            .set("stroke", "blue")
            .set("stroke-width", 2);

        main_group = main_group.add(placeholder);

        // Add a title
        let title = Text::new()
            .set("x", width / 2)
            .set("y", 30)
            .set("text-anchor", "middle")
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 16)
            .add(TextNode::new("Generated with Mapscow Mule"));

        main_group = main_group.add(title);

        document = document.add(main_group);

        // Write to file
        std::fs::write(output_path, document.to_string())?;
        Ok(())
    }
}
