use std::path::PathBuf;

/// Simple file dialog functionality
/// In a real implementation, you'd want to use rfd or a similar crate
pub struct FileDialog;

impl FileDialog {
    /// Show an open file dialog
    pub fn open_file(title: &str, filters: &[(&str, &[&str])]) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new().set_title(title);
        
        for (name, extensions) in filters {
            dialog = dialog.add_filter(*name, extensions);
        }
        
        dialog.pick_file()
    }
    
    /// Show a save file dialog
    pub fn save_file(title: &str, default_name: &str, filters: &[(&str, &[&str])]) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new()
            .set_title(title)
            .set_file_name(default_name);
        
        for (name, extensions) in filters {
            dialog = dialog.add_filter(*name, extensions);
        }
        
        dialog.save_file()
    }
    
    /// Show a folder selection dialog
    pub fn select_folder(title: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(title)
            .pick_folder()
    }
}

/// Predefined file filters for common use cases
pub struct FileFilters;

impl FileFilters {
    pub const OSM: (&'static str, &'static [&'static str]) = ("OpenStreetMap files", &["osm", "osm.xml"]);
    pub const GPX: (&'static str, &'static [&'static str]) = ("GPX files", &["gpx"]);
    pub const SVG: (&'static str, &'static [&'static str]) = ("SVG files", &["svg"]);
    pub const PNG: (&'static str, &'static [&'static str]) = ("PNG images", &["png"]);
    pub const JPEG: (&'static str, &'static [&'static str]) = ("JPEG images", &["jpg", "jpeg"]);
    pub const PDF: (&'static str, &'static [&'static str]) = ("PDF documents", &["pdf"]);
    pub const STYLESHEET: (&'static str, &'static [&'static str]) = ("Style files", &["yaml", "yml", "mss"]);
}
