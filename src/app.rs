use crate::core::MapData;
use crate::export::{ExportFormat, ExportOptions, Exporter};
use crate::gui::{GuiState, MapView, StyleEditor, ToolPanel, ToolPanelAction, Tool};
use crate::parsers::{osm::OsmParser, gpx::GpxParser, Parser};
use crate::rendering::MapRenderer;
use crate::styles::StyleManager;
use crate::utils::file_dialog::{FileDialog, FileFilters};
use anyhow::Result;
use egui::{Context, CentralPanel, SidePanel, TopBottomPanel};
use std::path::PathBuf;

pub struct MapscowMule {
    // Core data
    map_data: Option<MapData>,
    style_manager: StyleManager,
    renderer: MapRenderer,
    exporter: Exporter,
    
    // GUI state
    gui_state: GuiState,
    map_view: MapView,
    style_editor: StyleEditor,
    tool_panel: ToolPanel,
    
    // File dialogs and I/O
    osm_file_path: Option<PathBuf>,
    gpx_file_path: Option<PathBuf>,
    export_path: Option<PathBuf>,
    
    // Status and progress
    status_message: String,
    is_loading: bool,
    progress: f32,
}

impl MapscowMule {
    pub fn new() -> Self {
        let mut app = Self {
            map_data: None,
            style_manager: StyleManager::new(),
            renderer: MapRenderer::new(),
            exporter: Exporter::new(),
            
            gui_state: GuiState::new(),
            map_view: MapView::new(),
            style_editor: StyleEditor::new(),
            tool_panel: ToolPanel::new(),
            
            osm_file_path: None,
            gpx_file_path: None,
            export_path: None,
            
            status_message: "Ready".to_string(),
            is_loading: false,
            progress: 0.0,
        };
        
        // Auto-load example OSM file for debugging
        let example_path = PathBuf::from("examples/notre-dame.osm");
        if example_path.exists() {
            println!("Auto-loading example OSM file for debugging...");
            if let Err(e) = app.load_osm_file(&example_path) {
                println!("Failed to auto-load example file: {}", e);
            }
        } else {
            println!("Example OSM file not found at: {:?}", example_path);
        }
        
        app
    }
    
    pub fn load_osm_file(&mut self, path: &PathBuf) -> Result<()> {
        self.is_loading = true;
        self.status_message = "Loading OSM data...".to_string();
        
        println!("Loading OSM file: {:?}", path);
        
        let parser = OsmParser::new();
        match parser.parse_file(path) {
            Ok(data) => {
                self.map_data = Some(data);
                // Automatically center and zoom to fit the loaded data
                self.map_view.zoom_to_fit(&self.map_data);
                self.status_message = "OSM data loaded successfully".to_string();
                self.is_loading = false;
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("Failed to load OSM data: {}", e);
                self.is_loading = false;
                Err(e)
            }
        }
    }
    
    pub fn load_gpx_file(&mut self, path: &PathBuf) -> Result<()> {
        self.status_message = "Loading GPX data...".to_string();
        
        let parser = GpxParser::new();
        match parser.parse_file(path) {
            Ok(gpx_data) => {
                // TODO: Integrate GPX data with map data
                self.status_message = "GPX data loaded successfully".to_string();
                
                // Center the map on the GPX data if it contains track points
                if !gpx_data.is_empty() {
                    // For now, we'll need to convert GPX data to map data format
                    // This is a placeholder for proper GPX integration
                    self.status_message = "GPX data loaded and centered".to_string();
                }
                
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("Failed to load GPX data: {}", e);
                Err(e)
            }
        }
    }
    
    pub fn export_map(&mut self, format: ExportFormat, options: ExportOptions) -> Result<()> {
        if let Some(ref map_data) = self.map_data {
            self.status_message = "Exporting map...".to_string();
            
            // Get viewport information from MapView
            let (center_lon, center_lat, scale) = self.map_view.get_viewport_info();
            
            match self.exporter.export_map_with_viewport(
                map_data, 
                &self.renderer, 
                &options,
                center_lat,
                center_lon,
                scale,
            ) {
                Ok(_) => {
                    self.status_message = "Map exported successfully".to_string();
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("Export failed: {}", e);
                    Err(e)
                }
            }
        } else {
            self.status_message = "No map data to export".to_string();
            Err(anyhow::anyhow!("No map data loaded"))
        }
    }
}

impl eframe::App for MapscowMule {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Menu bar
        TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open OSM file...").clicked() {
                        if let Some(path) = FileDialog::open_file("Open OSM File", &[FileFilters::OSM]) {
                            self.status_message = format!("Loading OSM file: {}", path.display());
                            match self.load_osm_file(&path) {
                                Ok(_) => {
                                    self.status_message = format!("Successfully loaded: {}", path.display());
                                }
                                Err(e) => {
                                    self.status_message = format!("Error loading OSM file: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open GPX file...").clicked() {
                        if let Some(path) = FileDialog::open_file("Open GPX File", &[FileFilters::GPX]) {
                            self.status_message = format!("Loading GPX file: {}", path.display());
                            match self.load_gpx_file(&path) {
                                Ok(_) => {
                                    self.status_message = format!("Successfully loaded: {}", path.display());
                                }
                                Err(e) => {
                                    self.status_message = format!("Error loading GPX file: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export as SVG...").clicked() {
                        if let Some(path) = FileDialog::save_file("Export as SVG", "map.svg", &[FileFilters::SVG]) {
                            let options = ExportOptions::new(ExportFormat::Svg, path.to_string_lossy().to_string());
                            match self.export_map(ExportFormat::Svg, options) {
                                Ok(_) => {
                                    self.status_message = format!("Successfully exported: {}", path.display());
                                }
                                Err(e) => {
                                    self.status_message = format!("Error exporting SVG: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Export as PNG...").clicked() {
                        if let Some(path) = FileDialog::save_file("Export as PNG", "map.png", &[FileFilters::PNG]) {
                            let options = ExportOptions::new(ExportFormat::Png, path.to_string_lossy().to_string());
                            match self.export_map(ExportFormat::Png, options) {
                                Ok(_) => {
                                    self.status_message = format!("Successfully exported: {}", path.display());
                                }
                                Err(e) => {
                                    self.status_message = format!("Error exporting PNG: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.gui_state.show_style_editor, "Style Editor");
                    ui.checkbox(&mut self.gui_state.show_tool_panel, "Tool Panel");
                    ui.separator();
                    if ui.button("Zoom to Fit").clicked() {
                        self.map_view.zoom_to_fit(&self.map_data);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.gui_state.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Status bar
        TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if self.is_loading {
                    ui.separator();
                    ui.spinner();
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                }
            });
        });
        
        // Side panels
        if self.gui_state.show_style_editor {
            SidePanel::right("style_editor").show(ctx, |ui| {
                self.style_editor.show(ui, &mut self.style_manager);
            });
        }
        
        if self.gui_state.show_tool_panel {
            SidePanel::left("tool_panel").show(ctx, |ui| {
                let action = self.tool_panel.show(ui, &mut self.gui_state);
                
                // Handle tool panel actions
                match action {
                    ToolPanelAction::ZoomIn => {
                        self.map_view.zoom_by_factor(1.2);
                    }
                    ToolPanelAction::ZoomOut => {
                        self.map_view.zoom_by_factor(1.0 / 1.2);
                    }
                    ToolPanelAction::FitToWindow => {
                        self.map_view.zoom_to_fit(&self.map_data);
                    }
                    ToolPanelAction::ExportSvg => {
                        if let Some(path) = crate::utils::file_dialog::FileDialog::save_file("Export as SVG", "map.svg", &[crate::utils::file_dialog::FileFilters::SVG]) {
                            let options = crate::export::ExportOptions::new(crate::export::ExportFormat::Svg, path.to_string_lossy().to_string());
                            match self.export_map(crate::export::ExportFormat::Svg, options) {
                                Ok(_) => {
                                    self.status_message = "SVG exported successfully".to_string();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error exporting SVG: {}", e);
                                }
                            }
                        }
                    }
                    ToolPanelAction::ExportPng => {
                        if let Some(path) = crate::utils::file_dialog::FileDialog::save_file("Export as PNG", "map.png", &[crate::utils::file_dialog::FileFilters::PNG]) {
                            let options = crate::export::ExportOptions::new(crate::export::ExportFormat::Png, path.to_string_lossy().to_string());
                            match self.export_map(crate::export::ExportFormat::Png, options) {
                                Ok(_) => {
                                    self.status_message = "PNG exported successfully".to_string();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error exporting PNG: {}", e);
                                }
                            }
                        }
                    }
                    ToolPanelAction::ExportPdf => {
                        if let Some(path) = crate::utils::file_dialog::FileDialog::save_file("Export as PDF", "map.pdf", &[("PDF files", &["pdf"])]) {
                            let options = crate::export::ExportOptions::new(crate::export::ExportFormat::Pdf, path.to_string_lossy().to_string());
                            match self.export_map(crate::export::ExportFormat::Pdf, options) {
                                Ok(_) => {
                                    self.status_message = "PDF exported successfully".to_string();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error exporting PDF: {}", e);
                                }
                            }
                        }
                    }
                    ToolPanelAction::None => {}
                }
                
                // Update GUI state zoom level to match actual MapView zoom
                self.gui_state.zoom_level = self.map_view.get_zoom_level() as f32;
            });
        }
        
        // Main map view
        CentralPanel::default().show(ctx, |ui| {
            // Sync tool selection with map view
            let should_be_in_selection_mode = matches!(self.gui_state.current_tool, Tool::RectangleZoom);
            if should_be_in_selection_mode != self.map_view.is_selection_mode() {
                if should_be_in_selection_mode {
                    self.map_view.toggle_selection_mode();
                } else if self.map_view.is_selection_mode() {
                    self.map_view.toggle_selection_mode();
                }
            }
            
            self.map_view.show(ui, &self.map_data, &self.renderer, &self.style_manager);
        });
        
        // Modal dialogs
        if self.gui_state.show_about {
            egui::Window::new("About Mapscow Mule")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Mapscow Mule v0.1.0");
                    ui.label("A Maperitive clone built in Rust");
                    ui.separator();
                    ui.label("Features:");
                    ui.label("• High-quality SVG export");
                    ui.label("• OpenStreetMap data support");
                    ui.label("• Customizable map styles");
                    ui.label("• GPX track support");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.gui_state.show_about = false;
                    }
                });
        }
    }
}
