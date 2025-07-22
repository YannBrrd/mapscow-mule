use crate::styles::loader::{StyleManager, MapStyle};
use crate::parsers::stylesheet::{StyleSheet, StyleRule, Color};
use crate::gui::map_view::SelectedElement;
use egui::{Ui, Color32, ScrollArea, Context, TextEdit};
use log::info;
use std::collections::HashMap;

/// Style editor panel for customizing map appearance
pub struct StyleEditor {
    selected_rule: Option<usize>,
    color_picker_open: bool,
    current_color: Color32,
    // TOML editor state
    toml_content: String,
    toml_error: Option<String>,
    selected_tab: StyleEditorTab,
    // For live preview
    has_unsaved_changes: bool,
    needs_validation: bool, // Track if content needs validation
    last_validation_successful: bool, // Track if last validation was successful
    // Color editing
    editing_colors: HashMap<String, Color32>,
    // Element selection integration
    jump_to_section: Option<String>,
    search_highlight: Option<String>,
    load_current_style_needed: bool,
    last_searched_element: Option<String>, // Track the last element we searched for
    selected_element_info: Option<SelectedElement>, // Store full selected element info
    // Search functionality
    search_query: String,
    search_results: Vec<SearchResult>,
    current_search_index: usize,
    search_is_case_sensitive: bool,
    search_use_regex: bool,
    // Replace functionality
    replace_query: String,
    show_replace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StyleEditorTab {
    TomlEditor,
    VisualEditor,
    ColorPalette,
}

#[derive(Debug, Clone)]
struct SearchResult {
    line_number: usize,
    column_start: usize,
    column_end: usize,
    line_content: String,
    match_text: String,
}

impl StyleEditor {
    pub fn new() -> Self {
        Self {
            selected_rule: None,
            color_picker_open: false,
            current_color: Color32::BLACK,
            toml_content: String::new(),
            toml_error: None,
            selected_tab: StyleEditorTab::TomlEditor,
            has_unsaved_changes: false,
            needs_validation: false,
            last_validation_successful: true,
            editing_colors: HashMap::new(),
            jump_to_section: None,
            search_highlight: None,
            load_current_style_needed: false,
            last_searched_element: None,
            selected_element_info: None,
            // Search functionality
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            search_is_case_sensitive: false,
            search_use_regex: false,
            // Replace functionality
            replace_query: String::new(),
            show_replace: false,
        }
    }
    
    /// Jump to a specific TOML section based on selected map element
    pub fn jump_to_element_style(&mut self, selected_element: &SelectedElement) {
        info!("jump_to_element_style called with element ID: {}", selected_element.element_id);
        self.selected_tab = StyleEditorTab::TomlEditor;
        let toml_section = selected_element.style_info.toml_section.clone();
        self.jump_to_section = Some(toml_section.clone());
        self.search_highlight = Some(toml_section.clone());
        self.last_searched_element = Some(format!("{} ({})", 
            selected_element.style_info.category, 
            selected_element.style_info.subcategory));
        self.selected_element_info = Some(selected_element.clone());
        
        info!("Style editor jumping to section: {}, stored element: {:?}", toml_section, self.selected_element_info.is_some());
        
        // Force load the current style if not already loaded
        if self.toml_content.is_empty() {
            self.load_current_style_needed = true;
        }
    }
    
    /// Show the style editor as a modal window
    pub fn show_modal(&mut self, ctx: &Context, is_open: &mut bool, style_manager: &mut StyleManager, gui_state: &mut crate::gui::GuiState) {
        egui::Window::new("🎨 Style Editor")
            .open(is_open)
            .default_width(900.0)
            .default_height(700.0)
            .min_width(700.0)
            .min_height(500.0)
            .resizable(true)
            .collapsible(false)
            .vscroll(false)
            .show(ctx, |ui| {
                // Load current style content if needed
                if self.toml_content.is_empty() || self.load_current_style_needed {
                    self.load_current_style(style_manager);
                    self.load_current_style_needed = false;
                }
                
                // Add a scroll area that captures scroll events for the entire modal content
                egui::ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        self.show_content(ui, style_manager, gui_state);
                    });
            });
    }
    
    /// Show the style editor content (can be used in modal or panel)
    pub fn show_content(&mut self, ui: &mut Ui, style_manager: &mut StyleManager, gui_state: &mut crate::gui::GuiState) {
        ui.horizontal(|ui| {
            ui.heading("🎨 Style Editor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.has_unsaved_changes {
                    ui.colored_label(egui::Color32::ORANGE, "● Unsaved changes");
                }
            });
        });
        ui.separator();
        
        // Show selected element info if available
        let selected_element_clone = self.selected_element_info.clone();
        if let Some(ref element) = selected_element_clone {
            info!("Displaying selected element info in style editor: {:?}", element.element_id);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "🎯 Selected Element:");
                    ui.separator();
                    
                    // Element type and ID
                    ui.label(format!("{} #{}", 
                        match element.element_type {
                            crate::gui::map_view::ElementType::Way => "Way",
                            crate::gui::map_view::ElementType::Node => "Node", 
                            crate::gui::map_view::ElementType::Relation => "Relation",
                        },
                        element.element_id
                    ));
                    
                    ui.separator();
                    
                    // Style category and subcategory
                    ui.colored_label(egui::Color32::YELLOW, "Type:");
                    ui.label(format!("{} → {}", element.style_info.category, element.style_info.subcategory));
                    
                    ui.separator();
                    
                    // TOML section
                    ui.colored_label(egui::Color32::GREEN, "Section:");
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("[{}]", element.style_info.toml_section))
                            .code()
                            .color(egui::Color32::WHITE)
                    ).selectable(true));
                    
                    // Clear button
                    if ui.small_button("Clear").clicked() {
                        self.selected_element_info = None;
                        self.search_highlight = None;
                        self.jump_to_section = None;
                        self.last_searched_element = None;
                    }
                });
                
                // Show element tags if available (collapsible)
                if !element.tags.is_empty() {
                    ui.collapsing("🏷️ Element Tags", |ui| {
                        egui::Grid::new("element_tags")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                for (key, value) in &element.tags {
                                    ui.monospace(key);
                                    ui.monospace(value);
                                    ui.end_row();
                                }
                            });
                    });
                }
            });
            ui.separator();
        }
        
        // Tab selector
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, StyleEditorTab::TomlEditor, "📝 TOML Editor");
            ui.selectable_value(&mut self.selected_tab, StyleEditorTab::ColorPalette, "🎨 Color Palette");
            ui.selectable_value(&mut self.selected_tab, StyleEditorTab::VisualEditor, "🔧 Visual Editor");
        });
        ui.separator();
        
        // Tab content
        match self.selected_tab {
            StyleEditorTab::TomlEditor => self.show_toml_editor(ui, style_manager, gui_state),
            StyleEditorTab::ColorPalette => self.show_color_palette_editor(ui, style_manager),
            StyleEditorTab::VisualEditor => self.show_visual_editor(ui, style_manager),
        }
        
        ui.separator();
        
        // Action buttons
        ui.horizontal(|ui| {
            // Gray out save button if we have unsaved changes that need validation
            let can_save = !self.has_unsaved_changes || (self.has_unsaved_changes && !self.needs_validation);
            
            if ui.add_enabled(can_save, egui::Button::new("💾 Save Style")).clicked() {
                self.save_style(style_manager);
            }
            
            if ui.button("� Load Style File").clicked() {
                self.load_style_file();
            }
            
            if ui.button("� Reload").clicked() {
                self.load_current_style(style_manager);
            }
            
            if ui.button("� Export TOML").clicked() {
                self.export_toml();
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    // Reset state when closing
                    self.toml_content.clear();
                    self.has_unsaved_changes = false;
                }
            });
        });
    }
    
    /// Legacy method for backward compatibility (now shows content only)
    pub fn show(&mut self, ui: &mut Ui, style_manager: &mut StyleManager, gui_state: &mut crate::gui::GuiState) {
        self.show_content(ui, style_manager, gui_state);
    }
    
    fn show_stylesheet_editor(&mut self, ui: &mut Ui, stylesheet: &mut StyleSheet) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ Add Rule").clicked() {
                stylesheet.rules.push(StyleRule {
                    selectors: vec![],
                    style: Default::default(),
                });
                self.selected_rule = Some(stylesheet.rules.len() - 1);
            }
            
            if ui.button("💾 Save Stylesheet").clicked() {
                // TODO: Implement save functionality
            }
            
            if ui.button("📁 Load Stylesheet").clicked() {
                // TODO: Implement load functionality
            }
        });
        
        ui.separator();
        
        // Rules list and editor
        ui.horizontal(|ui| {
            // Rules list (left side)
            ui.vertical(|ui| {
                ui.heading("Rules");
                ui.set_width(200.0);
                
                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (index, rule) in stylesheet.rules.iter().enumerate() {
                            let rule_name = self.get_rule_display_name(rule);
                            let selected = self.selected_rule == Some(index);
                            
                            if ui.selectable_label(selected, rule_name).clicked() {
                                self.selected_rule = Some(index);
                            }
                        }
                    });
                
                // Delete button
                if let Some(selected) = self.selected_rule {
                    if ui.button("🗑 Delete Rule").clicked() && selected < stylesheet.rules.len() {
                        stylesheet.rules.remove(selected);
                        self.selected_rule = None;
                    }
                }
            });
            
            ui.separator();
            
            // Rule editor (right side)
            ui.vertical(|ui| {
                if let Some(selected) = self.selected_rule {
                    if let Some(rule) = stylesheet.rules.get_mut(selected) {
                        self.show_rule_editor(ui, rule);
                    }
                } else {
                    ui.label("Select a rule to edit");
                }
            });
        });
    }
    
    fn show_rule_editor(&mut self, ui: &mut Ui, rule: &mut StyleRule) {
        ui.heading("Rule Editor");
        
        // Selectors section
        ui.group(|ui| {
            ui.label("Selectors:");
            
            for (index, selector) in rule.selectors.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    match selector {
                        crate::parsers::stylesheet::FeatureSelector::Tag { key, value } => {
                            ui.label("Tag:");
                            ui.text_edit_singleline(key);
                            ui.label("=");
                            
                            let mut value_str = value.clone().unwrap_or_default();
                            ui.text_edit_singleline(&mut value_str);
                            *value = if value_str.is_empty() { None } else { Some(value_str) };
                        }
                        _ => {
                            ui.label("Other selector type");
                        }
                    }
                    
                    if ui.button("❌").clicked() {
                        // Mark for removal
                    }
                });
            }
            
            if ui.button("➕ Add Selector").clicked() {
                rule.selectors.push(crate::parsers::stylesheet::FeatureSelector::Tag {
                    key: "key".to_string(),
                    value: Some("value".to_string()),
                });
            }
        });
        
        ui.separator();
        
        // Style properties section
        ui.group(|ui| {
            ui.label("Style Properties:");
            
            // Draw mode
            ui.horizontal(|ui| {
                ui.label("Draw mode:");
                egui::ComboBox::from_id_salt("draw_mode")
                    .selected_text(format!("{:?}", rule.style.draw_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut rule.style.draw_mode, 
                            crate::parsers::stylesheet::DrawMode::Line, "Line");
                        ui.selectable_value(&mut rule.style.draw_mode, 
                            crate::parsers::stylesheet::DrawMode::Fill, "Fill");
                        ui.selectable_value(&mut rule.style.draw_mode, 
                            crate::parsers::stylesheet::DrawMode::Both, "Both");
                        ui.selectable_value(&mut rule.style.draw_mode, 
                            crate::parsers::stylesheet::DrawMode::Point, "Point");
                    });
            });
            
            // Line color
            ui.horizontal(|ui| {
                ui.label("Line color:");
                if let Some(ref mut color) = rule.style.line_color {
                    let mut rgb = [
                        color.r as f32 / 255.0,
                        color.g as f32 / 255.0,
                        color.b as f32 / 255.0
                    ];
                    if ui.color_edit_button_rgb(&mut rgb).changed() {
                        color.r = (rgb[0] * 255.0) as u8;
                        color.g = (rgb[1] * 255.0) as u8;
                        color.b = (rgb[2] * 255.0) as u8;
                    }
                } else {
                    if ui.button("Set Line Color").clicked() {
                        rule.style.line_color = Some(Color::new(0, 0, 0, 255));
                    }
                }
                
                if rule.style.line_color.is_some() {
                    if ui.button("❌").clicked() {
                        rule.style.line_color = None;
                    }
                }
            });
            
            // Fill color
            ui.horizontal(|ui| {
                ui.label("Fill color:");
                if let Some(ref mut color) = rule.style.fill_color {
                    let mut rgb = [
                        color.r as f32 / 255.0,
                        color.g as f32 / 255.0,
                        color.b as f32 / 255.0
                    ];
                    if ui.color_edit_button_rgb(&mut rgb).changed() {
                        color.r = (rgb[0] * 255.0) as u8;
                        color.g = (rgb[1] * 255.0) as u8;
                        color.b = (rgb[2] * 255.0) as u8;
                    }
                } else {
                    if ui.button("Set Fill Color").clicked() {
                        rule.style.fill_color = Some(Color::new(128, 128, 128, 255));
                    }
                }
                
                if rule.style.fill_color.is_some() {
                    if ui.button("❌").clicked() {
                        rule.style.fill_color = None;
                    }
                }
            });
            
            // Line width
            ui.horizontal(|ui| {
                ui.label("Line width:");
                ui.add(egui::Slider::new(&mut rule.style.line_width, 0.0..=10.0));
            });
            
            // Font properties
            ui.horizontal(|ui| {
                ui.label("Font size:");
                ui.add(egui::Slider::new(&mut rule.style.font_size, 6.0..=72.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("Font family:");
                if let Some(ref mut font) = rule.style.font_family {
                    ui.text_edit_singleline(font);
                } else {
                    if ui.button("Set Font").clicked() {
                        rule.style.font_family = Some("Arial".to_string());
                    }
                }
            });
            
            // Text field
            ui.horizontal(|ui| {
                ui.label("Text field:");
                if let Some(ref mut text_field) = rule.style.text_field {
                    ui.text_edit_singleline(text_field);
                } else {
                    if ui.button("Set Text Field").clicked() {
                        rule.style.text_field = Some("name".to_string());
                    }
                }
                
                if rule.style.text_field.is_some() {
                    if ui.button("❌").clicked() {
                        rule.style.text_field = None;
                    }
                }
            });
            
            // Zoom range
            ui.horizontal(|ui| {
                ui.label("Min zoom:");
                if let Some(ref mut min_zoom) = rule.style.min_zoom {
                    ui.add(egui::Slider::new(min_zoom, 0..=20));
                } else {
                    if ui.button("Set Min Zoom").clicked() {
                        rule.style.min_zoom = Some(0);
                    }
                }
                
                if rule.style.min_zoom.is_some() {
                    if ui.button("❌").clicked() {
                        rule.style.min_zoom = None;
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Max zoom:");
                if let Some(ref mut max_zoom) = rule.style.max_zoom {
                    ui.add(egui::Slider::new(max_zoom, 0..=20));
                } else {
                    if ui.button("Set Max Zoom").clicked() {
                        rule.style.max_zoom = Some(18);
                    }
                }
                
                if rule.style.max_zoom.is_some() {
                    if ui.button("❌").clicked() {
                        rule.style.max_zoom = None;
                    }
                }
            });
        });
    }
    
    fn get_rule_display_name(&self, rule: &StyleRule) -> String {
        if rule.selectors.is_empty() {
            "Empty Rule".to_string()
        } else {
            let selector = &rule.selectors[0];
            match selector {
                crate::parsers::stylesheet::FeatureSelector::Tag { key, value } => {
                    if let Some(v) = value {
                        format!("{}={}", key, v)
                    } else {
                        key.clone()
                    }
                }
                crate::parsers::stylesheet::FeatureSelector::ElementType(element_type) => {
                    format!("{:?}", element_type)
                }
                crate::parsers::stylesheet::FeatureSelector::ZoomRange { min, max } => {
                    format!("Zoom {}-{}", 
                        min.map(|z| z.to_string()).unwrap_or("*".to_string()),
                        max.map(|z| z.to_string()).unwrap_or("*".to_string())
                    )
                }
            }
        }
    }
    
    /// Load current style into TOML editor
    fn load_current_style(&mut self, style_manager: &StyleManager) {
        // Try to load the current style's TOML content
        if let Some(current_style_name) = style_manager.get_available_styles().first() {
            let style_path = format!("assets/styles/{}.toml", current_style_name);
            match std::fs::read_to_string(&style_path) {
                Ok(content) => {
                    self.toml_content = content;
                    self.toml_error = None;
                    self.has_unsaved_changes = false;
                }
                Err(e) => {
                    self.toml_error = Some(format!("Failed to load style file: {}", e));
                    self.toml_content = self.generate_default_toml();
                }
            }
        } else {
            self.toml_content = self.generate_default_toml();
        }
    }
    
    /// Show TOML editor tab
    fn show_toml_editor(&mut self, ui: &mut Ui, style_manager: &mut StyleManager, gui_state: &mut crate::gui::GuiState) {
        // Auto-load current style if needed
        if self.load_current_style_needed {
            self.load_current_style(style_manager);
            self.load_current_style_needed = false;
        }
        
        ui.horizontal(|ui| {
            ui.label("Current Style:");
            let available_styles: Vec<String> = style_manager.get_available_styles().iter().map(|s| s.to_string()).collect();
            
            // Use the current style from gui_state, fallback to first available
            if !available_styles.contains(&gui_state.selected_style) {
                if let Some(first_style) = available_styles.first() {
                    gui_state.selected_style = first_style.clone();
                }
            }
            
            let current_display_name = match gui_state.selected_style.as_str() {
                "google-maps" => "Google Maps".to_string(),
                "osm-default" => "OSM Default".to_string(),
                name => name.replace('-', " ").replace('_', " ")
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            
            egui::ComboBox::from_id_salt("toml_style_selector")
                .selected_text(current_display_name)
                .show_ui(ui, |ui| {
                    for style_name in &available_styles {
                        let display_name = match style_name.as_str() {
                            "google-maps" => "Google Maps".to_string(),
                            "osm-default" => "OSM Default".to_string(),
                            name => name.replace('-', " ").replace('_', " ")
                                .split_whitespace()
                                .map(|word| {
                                    let mut chars = word.chars();
                                    match chars.next() {
                                        None => String::new(),
                                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(" ")
                        };
                        
                        if ui.selectable_value(&mut gui_state.selected_style, style_name.clone(), display_name).clicked() {
                            if let Err(e) = style_manager.load_style(style_name) {
                                self.toml_error = Some(format!("Error loading style: {}", e));
                            } else {
                                self.load_current_style(style_manager);
                                // Clear any section highlighting when switching styles
                                self.search_highlight = None;
                                self.jump_to_section = None;
                                self.last_searched_element = None;
                                self.selected_element_info = None;
                            }
                        }
                    }
                });
            
            if ui.button("🔄 Validate").clicked() {
                self.validate_toml();
                self.needs_validation = false;
                // Set validation success based on whether there's a TOML error
                self.last_validation_successful = self.toml_error.is_none();
            }
            
        // Show section status banner
        let search_section_clone = self.search_highlight.clone();
        let element_info_clone = self.last_searched_element.clone();
        
        if let Some(ref section) = search_section_clone {
            ui.separator();
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::LIGHT_BLUE, "📍 Searching for:");
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("[{}]", section))
                        .code()
                        .color(egui::Color32::WHITE)
                ).selectable(true));
                
                if let Some(ref element_info) = element_info_clone {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, "🎯 Element:");
                    ui.label(element_info);
                }
                
                if ui.button("Clear").clicked() {
                    self.search_highlight = None;
                    self.jump_to_section = None;
                    self.last_searched_element = None;
                    self.selected_element_info = None;
                }
            });
        }
        });
        
        // Show error if any
        if let Some(ref error) = self.toml_error {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }
        
        ui.separator();
        
        // Handle jumping to section
        if let Some(ref section) = self.jump_to_section.take() {
            if let Some(pos) = self.find_section_in_content(section) {
                // Calculate the line number for better user experience
                let line_num = self.toml_content[..pos].matches('\n').count() + 1;
                info!("Found section '{}' at position {} (line {})", section, pos, line_num);
                self.search_highlight = Some(section.clone());
            } else {
                // Add helpful content if section doesn't exist
                self.highlight_section_in_content(section);
                self.search_highlight = Some(section.clone());
            }
        }
        
        // TOML text editor with highlighting
        let text_edit_response = ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                // Enhanced search functionality
                ui.vertical(|ui| {
                    // Section highlighting (from element selection)
                    let search_highlight_clone = self.search_highlight.clone();
                    if let Some(ref highlight) = search_highlight_clone {
                        ui.horizontal(|ui| {
                            ui.label("🎯 Section:");
                            ui.monospace(highlight);
                            
                            // Show found section info
                            if let Some(pos) = self.find_section_in_content(highlight) {
                                let line_num = self.toml_content[..pos].matches('\n').count() + 1;
                                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("✓ Found at line {}", line_num));
                            } else {
                                ui.colored_label(egui::Color32::ORANGE, "⚠ Section not found");
                                ui.label("→ You can add this section manually to customize styling");
                            }
                            
                            if ui.button("Clear").clicked() {
                                self.search_highlight = None;
                            }
                        });
                    }
                    
                    // Text search interface
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🔍 Search:");
                            
                            let search_response = ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Search in TOML content...")
                                    .desired_width(200.0)
                            );
                            
                            if search_response.changed() {
                                self.perform_search();
                            }
                            
                            // Toggle replace panel
                            if ui.button("🔄").on_hover_text("Show replace options").clicked() {
                                self.show_replace = !self.show_replace;
                            }
                            
                            // Search options
                            ui.checkbox(&mut self.search_is_case_sensitive, "Case sensitive");
                            
                            // Search navigation
                            if !self.search_results.is_empty() {
                                ui.separator();
                                ui.label(format!("{}/{}", self.current_search_index + 1, self.search_results.len()));
                                
                                if ui.button("⬆").on_hover_text("Previous result (Shift+F3)").clicked() {
                                    self.prev_search_result();
                                }
                                
                                if ui.button("⬇").on_hover_text("Next result (F3)").clicked() {
                                    self.next_search_result();
                                }
                                
                                // Show current result info
                                if let Some(result) = self.get_current_search_result() {
                                    ui.separator();
                                    ui.colored_label(egui::Color32::LIGHT_GREEN, 
                                        format!("Line {}: '{}'", result.line_number, result.match_text));
                                }
                            } else if !self.search_query.is_empty() {
                                ui.separator();
                                ui.colored_label(egui::Color32::YELLOW, "No results found");
                            }
                            
                            // Clear search
                            if !self.search_query.is_empty() && ui.button("✖").on_hover_text("Clear search").clicked() {
                                self.search_query.clear();
                                self.search_results.clear();
                                self.current_search_index = 0;
                                self.show_replace = false;
                            }
                        });
                        
                        // Replace interface (collapsible)
                        if self.show_replace {
                            ui.horizontal(|ui| {
                                ui.label("↻ Replace:");
                                
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.replace_query)
                                        .hint_text("Replace with...")
                                        .desired_width(200.0)
                                );
                                
                                // Replace buttons
                                if !self.search_results.is_empty() {
                                    if ui.button("Replace").on_hover_text("Replace current result").clicked() {
                                        self.replace_current();
                                    }
                                    
                                    if ui.button("Replace All").on_hover_text("Replace all occurrences").clicked() {
                                        self.replace_all();
                                    }
                                }
                            });
                        }
                    });
                });
                
                // Create a text editor with better highlighting
                let mut text_edit = TextEdit::multiline(&mut self.toml_content)
                    .code_editor()
                    .desired_rows(20)
                    .font(egui::TextStyle::Monospace);
                
                // Note: Can't set cursor position due to borrow checker limitations
                // The highlighting info is shown above the editor instead
                
                ui.add_sized(
                    [ui.available_width(), 400.0],
                    text_edit
                )
            });
        
        if text_edit_response.inner.changed() {
            self.has_unsaved_changes = true;
            self.needs_validation = true;
            // Clear any previous validation state since content changed
            self.toml_error = None;
        }
        
        // Handle keyboard shortcuts for search
        if ui.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            // Focus search (Ctrl+F) - for now just highlight that search is available
            // In a real implementation, you'd focus the search text field
        }
        
        if ui.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) {
            // Show replace (Ctrl+H)
            self.show_replace = !self.show_replace;
        }
        
        if !self.search_results.is_empty() {
            if ui.input(|i| i.key_pressed(egui::Key::F3)) {
                self.next_search_result();
            }
            if ui.input(|i| i.key_pressed(egui::Key::F3) && i.modifiers.shift) {
                self.prev_search_result();
            }
        }
        
        // Quick navigation buttons for common sections
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick jump:");
            
            let sections = vec![
                ("🏠 Buildings", "buildings"),
                ("🛣️ Roads", "roads"),
                ("🌊 Water", "waterways"),
                ("🏞️ Areas", "areas"),
                ("🚂 Railways", "railways"),
                ("📍 POIs", "pois"),
                ("🎨 Background", "background"),
            ];
            
            for (label, section) in sections {
                if ui.small_button(label).clicked() {
                    self.jump_to_section = Some(section.to_string());
                    self.search_highlight = Some(section.to_string());
                }
            }
        });
        
        // Quick search patterns
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick search:");
            
            let patterns = vec![
                ("🎨 Colors", "#"),
                ("📏 Widths", "width"),
                ("🔢 Numbers", r"\d+"),
                ("⚪ Borders", "border"),
                ("📝 Fonts", "font"),
                ("🔧 Config", "="),
            ];
            
            for (label, pattern) in patterns {
                if ui.small_button(label).clicked() {
                    self.search_query = pattern.to_string();
                    if pattern.contains(r"\d") {
                        self.search_use_regex = true;
                    }
                    self.perform_search();
                }
            }
        });
        
        // TOML help section
        ui.separator();
        ui.collapsing("📚 TOML Reference", |ui| {
            ui.label("Common color formats:");
            ui.monospace("color = \"#FF0000\"  # Red");
            ui.monospace("color = \"#00FF00\"  # Green");
            ui.monospace("color = \"#0000FF\"  # Blue");
            ui.separator();
            ui.label("Example road configuration:");
            ui.monospace("[roads.primary]");
            ui.monospace("color = \"#FFFFFF\"");
            ui.monospace("width = 3");
            ui.monospace("border_color = \"#B4C4D1\"");
            ui.monospace("border_width = 1");
        });
    }
    
    /// Find a section in the TOML content and return its position
    fn find_section_in_content(&self, section: &str) -> Option<usize> {
        info!("Searching for section: '{}'", section);
        
        // Look for exact section headers first
        let exact_patterns = vec![
            format!("[{}]", section),
            format!("[{}.", section), // For parent sections like [roads.
        ];
        
        for pattern in &exact_patterns {
            if let Some(pos) = self.toml_content.find(pattern) {
                info!("Found exact section '{}' with pattern '{}' at position {}", section, pattern, pos);
                return Some(pos);
            }
        }
        
        // Handle specific mappings based on our element categorization
        let mapped_section = match section {
            // Road sections
            s if s.starts_with("roads.") => {
                let road_type = s.strip_prefix("roads.").unwrap_or("");
                format!("[roads.{}]", road_type)
            }
            // POI sections  
            s if s.starts_with("pois.") => {
                let poi_type = s.strip_prefix("pois.").unwrap_or("");
                format!("[pois.{}]", poi_type)
            }
            // Top-level sections
            "buildings" => "[buildings]".to_string(),
            "landuse" => "[landuse]".to_string(),
            "natural" => "[natural]".to_string(),
            "water" => "[water]".to_string(),
            "leisure" => "[leisure]".to_string(),
            "railway" => "[railway]".to_string(),
            "aeroway" => "[aeroway]".to_string(),
            "boundaries" => "[boundaries]".to_string(),
            _ => format!("[{}]", section),
        };
        
        if let Some(pos) = self.toml_content.find(&mapped_section) {
            info!("Found mapped section '{}' -> '{}' at position {}", section, mapped_section, pos);
            return Some(pos);
        }
        
        // Try alternative section names for backwards compatibility
        let alt_sections = match section {
            s if s.starts_with("highways.") || s.starts_with("roads.") => {
                let road_type = s.split('.').last().unwrap_or("");
                vec![
                    format!("[roads.{}]", road_type),
                    format!("[highway.{}]", road_type),
                    format!("[highways.{}]", road_type),
                ]
            }
            "buildings" => vec!["[building]".to_string()],
            "waterways" | "water" => vec!["[water]".to_string(), "[waterway]".to_string()],
            "railways" | "railway" => vec!["[railway]".to_string(), "[rail]".to_string()],
            _ => vec![],
        };
        
        for alt_section in alt_sections {
            if let Some(pos) = self.toml_content.find(&alt_section) {
                info!("Found alternative section '{}' for '{}' at position {}", alt_section, section, pos);
                return Some(pos);
            }
        }
        
        // If still not found, try a simple substring search for the section name
        let simple_name = section.split('.').last().unwrap_or(section);
        if let Some(pos) = self.toml_content.find(&format!("[{}]", simple_name)) {
            info!("Found simple section '{}' for '{}' at position {}", simple_name, section, pos);
            return Some(pos);
        }
        
        info!("Section '{}' not found in TOML content", section);
        None
    }
    
    /// Highlight a section in the content by adding a comment or finding existing section
    fn highlight_section_in_content(&mut self, section: &str) {
        if let Some(pos) = self.find_section_in_content(section) {
            info!("Highlighting section '{}' found at position {}", section, pos);
            // The section exists, no need to add anything
        } else {
            info!("Section '{}' not found, suggesting where to add it", section);
            
            // Add helpful comment suggesting where to add the section
            let suggestion = match section {
                s if s.starts_with("highways.") => {
                    let highway_type = s.strip_prefix("highways.").unwrap_or("");
                    format!("\n# Add highway styling for '{}' here:\n# [roads.{}]\n# color = \"#FFFFFF\"\n# width = 2\n# border_color = \"#CCCCCC\"\n# border_width = 1\n", highway_type, highway_type)
                }
                "buildings" => {
                    "\n# Add building styling here:\n# [buildings]\n# color = \"#E0E0E0\"\n# border_color = \"#CCCCCC\"\n# border_width = 1\n".to_string()
                }
                _ => {
                    format!("\n# Add styling for '{}' here:\n# [{}]\n# color = \"#FFFFFF\"\n", section, section)
                }
            };
            
            // Add the suggestion at the end if not already present
            if !self.toml_content.contains(&format!("Add styling for '{}'", section)) &&
               !self.toml_content.contains(&format!("Add highway styling for")) {
                self.toml_content.push_str(&suggestion);
                self.has_unsaved_changes = true;
            }
        }
    }
    
    /// Show color palette editor tab
    fn show_color_palette_editor(&mut self, ui: &mut Ui, _style_manager: &mut StyleManager) {
        ui.label("🎨 Quick Color Editor");
        ui.separator();
        
        // Parse current colors from TOML
        let colors = self.extract_colors_from_toml();
        
        ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    // Left column - color categories
                    columns[0].heading("Categories");
                    
                    if columns[0].button("🌊 Water Colors").clicked() {
                        // Focus on water colors
                    }
                    if columns[0].button("🛣️ Road Colors").clicked() {
                        // Focus on road colors  
                    }
                    if columns[0].button("🏢 Building Colors").clicked() {
                        // Focus on building colors
                    }
                    if columns[0].button("🌳 Nature Colors").clicked() {
                        // Focus on nature colors
                    }
                    if columns[0].button("📍 POI Colors").clicked() {
                        // Focus on POI colors
                    }
                    
                    // Right column - color editors
                    columns[1].heading("Colors");
                    
                    for (category, category_colors) in colors {
                        columns[1].collapsing(&category, |ui| {
                            for (key, color_str) in category_colors {
                                ui.horizontal(|ui| {
                                    ui.label(&key);
                                    
                                    if let Ok(color) = self.parse_color(&color_str) {
                                        let mut rgb = [
                                            color.r() as f32 / 255.0,
                                            color.g() as f32 / 255.0,
                                            color.b() as f32 / 255.0,
                                        ];
                                        
                                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                                            let new_color = Color32::from_rgb(
                                                (rgb[0] * 255.0) as u8,
                                                (rgb[1] * 255.0) as u8,
                                                (rgb[2] * 255.0) as u8,
                                            );
                                            let hex_color = format!("#{:02X}{:02X}{:02X}", 
                                                new_color.r(), new_color.g(), new_color.b());
                                            self.update_color_in_toml(&key, &hex_color);
                                            self.has_unsaved_changes = true;
                                        }
                                    }
                                });
                            }
                        });
                    }
                });
            });
    }
    
    /// Show visual/GUI editor tab  
    fn show_visual_editor(&mut self, ui: &mut Ui, _style_manager: &mut StyleManager) {
        ui.label("🔧 Visual Style Editor");
        ui.separator();
        
        // This is the existing stylesheet editor functionality
        ui.label("Advanced visual editing coming soon...");
        ui.separator();
        ui.label("This will provide:");
        ui.label("• Visual preview of style changes");
        ui.label("• Drag-and-drop color editing");
        ui.label("• Layer-based style management");
        ui.label("• Real-time map preview");
    }
    
    /// Validate the current TOML content
    fn validate_toml(&mut self) {
        match toml::from_str::<MapStyle>(&self.toml_content) {
            Ok(_) => {
                self.toml_error = None;
            }
            Err(e) => {
                self.toml_error = Some(format!("TOML syntax error: {}", e));
            }
        }
    }
    
    /// Save the current style
    fn save_style(&mut self, style_manager: &mut StyleManager) {
        // First validate
        self.validate_toml();
        
        if self.toml_error.is_some() {
            return;
        }
        
        // Get current style name by collecting to owned strings
        let available_styles: Vec<String> = style_manager.get_available_styles().iter().map(|s| s.to_string()).collect();
        let current_style_name = available_styles.first().cloned();
        
        // Save to file
        if let Some(style_name) = current_style_name {
            let style_path = format!("assets/styles/{}.toml", style_name);
            match std::fs::write(&style_path, &self.toml_content) {
                Ok(_) => {
                    self.has_unsaved_changes = false;
                    self.needs_validation = false;
                    self.last_validation_successful = true;
                    // Reload the style in the manager
                    if let Err(e) = style_manager.load_style(&style_name) {
                        self.toml_error = Some(format!("Failed to reload style: {}", e));
                    }
                }
                Err(e) => {
                    self.toml_error = Some(format!("Failed to save file: {}", e));
                }
            }
        }
    }
    
    /// Load style from file
    fn load_style_file(&mut self) {
        // TODO: Implement file dialog
        // For now, just reload current
        self.toml_error = Some("File dialog not implemented yet".to_string());
    }
    
    /// Export TOML to clipboard
    fn export_toml(&mut self) {
        // TODO: Copy to clipboard
        self.toml_error = Some("Export to clipboard not implemented yet".to_string());
    }
    
    /// Generate default TOML content
    fn generate_default_toml(&self) -> String {
        let mut content = String::new();
        content.push_str("# Default Map Style Configuration\n\n");
        content.push_str("[background]\n");
        content.push_str("color = \"");
        content.push_str("#F2F1EC");
        content.push_str("\"\n\n");
        content.push_str("[water]\n");
        content.push_str("color = \"");
        content.push_str("#AAD3DF");
        content.push_str("\"\n");
        content.push_str("opacity = 1.0\n\n");
        content.push_str("[landuse]\n");
        content.push_str("forest = \"");
        content.push_str("#C8D5B9");
        content.push_str("\"\n");
        content.push_str("residential = \"");
        content.push_str("#F2F1EC");
        content.push_str("\"\n");
        content.push_str("commercial = \"");
        content.push_str("#F2F1EC");
        content.push_str("\"\n");
        content.push_str("industrial = \"");
        content.push_str("#E8E7E2");
        content.push_str("\"\n\n");
        content.push_str("[leisure]\n");
        content.push_str("park = \"");
        content.push_str("#B8D2A0");
        content.push_str("\"\n");
        content.push_str("playground = \"");
        content.push_str("#B8D2A0");
        content.push_str("\"\n\n");
        content.push_str("[buildings]\n");
        content.push_str("fill = \"");
        content.push_str("#EAEAE8");
        content.push_str("\"\n");
        content.push_str("stroke = \"");
        content.push_str("#D8D8D6");
        content.push_str("\"\n");
        content.push_str("stroke_width = 0.3\n\n");
        content.push_str("[roads.primary]\n");
        content.push_str("color = \"");
        content.push_str("#FFFFFF");
        content.push_str("\"\n");
        content.push_str("width = 3\n");
        content.push_str("border_color = \"");
        content.push_str("#B4C4D1");
        content.push_str("\"\n");
        content.push_str("border_width = 1\n\n");
        content.push_str("[pois.default]\n");
        content.push_str("color = \"");
        content.push_str("#95a5a6");
        content.push_str("\"\n");
        content.push_str("radius = 3\n");
        content
    }
    
    /// Extract colors from TOML for color palette editor
    fn extract_colors_from_toml(&self) -> HashMap<String, Vec<(String, String)>> {
        let mut colors = HashMap::new();
        
        // Parse the TOML and extract color values
        if let Ok(parsed) = toml::from_str::<toml::Value>(&self.toml_content) {
            if let toml::Value::Table(table) = parsed {
                for (section_name, section_value) in table {
                    if let toml::Value::Table(section_table) = section_value {
                        let mut section_colors = Vec::new();
                        
                        for (key, value) in section_table {
                            match value {
                                toml::Value::String(s) if s.starts_with('#') => {
                                    section_colors.push((key, s));
                                }
                                toml::Value::Table(subtable) => {
                                    for (subkey, subvalue) in subtable {
                                        if let toml::Value::String(s) = subvalue {
                                            if s.starts_with('#') {
                                                section_colors.push((format!("{}.{}", key, subkey), s));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        
                        if !section_colors.is_empty() {
                            colors.insert(section_name, section_colors);
                        }
                    }
                }
            }
        }
        
        colors
    }
    
    /// Parse a hex color string to Color32
    fn parse_color(&self, color_str: &str) -> Result<Color32, ()> {
        if !color_str.starts_with('#') || color_str.len() != 7 {
            return Err(());
        }
        
        let r = u8::from_str_radix(&color_str[1..3], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&color_str[3..5], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&color_str[5..7], 16).map_err(|_| ())?;
        
        Ok(Color32::from_rgb(r, g, b))
    }
    
    /// Update a color value in the TOML content
    fn update_color_in_toml(&mut self, key: &str, new_color: &str) {
        // Simple string replacement for now
        // TODO: Implement proper TOML parsing and modification
        let search_pattern = format!("{} = ", key);
        let lines: Vec<&str> = self.toml_content.lines().collect();
        let mut new_lines = Vec::new();
        
        for line in lines {
            if line.contains(&search_pattern) && line.contains('#') {
                // Replace the color value
                if let Some(equals_pos) = line.find('=') {
                    let prefix = &line[..equals_pos + 1];
                    new_lines.push(format!("{} \"{}\"", prefix, new_color));
                } else {
                    new_lines.push(line.to_string());
                }
            } else {
                new_lines.push(line.to_string());
            }
        }
        
        self.toml_content = new_lines.join("\n");
    }
    
    /// Perform search in TOML content
    fn perform_search(&mut self) {
        self.search_results.clear();
        self.current_search_index = 0;
        
        if self.search_query.is_empty() {
            return;
        }
        
        let lines: Vec<&str> = self.toml_content.lines().collect();
        
        for (line_index, line) in lines.iter().enumerate() {
            let search_line = if self.search_is_case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };
            
            let search_query = if self.search_is_case_sensitive {
                self.search_query.clone()
            } else {
                self.search_query.to_lowercase()
            };
            
            if self.search_use_regex {
                // Simple regex search (would need regex crate for full support)
                if search_line.contains(&search_query) {
                    if let Some(start) = search_line.find(&search_query) {
                        let end = start + search_query.len();
                        self.search_results.push(SearchResult {
                            line_number: line_index + 1,
                            column_start: start,
                            column_end: end,
                            line_content: line.to_string(),
                            match_text: search_query.clone(),
                        });
                    }
                }
            } else {
                // Simple text search
                let mut start_pos = 0;
                while let Some(pos) = search_line[start_pos..].find(&search_query) {
                    let actual_pos = start_pos + pos;
                    let end_pos = actual_pos + search_query.len();
                    
                    self.search_results.push(SearchResult {
                        line_number: line_index + 1,
                        column_start: actual_pos,
                        column_end: end_pos,
                        line_content: line.to_string(),
                        match_text: line[actual_pos..end_pos].to_string(),
                    });
                    
                    start_pos = actual_pos + 1;
                }
            }
        }
    }
    
    /// Navigate to next search result
    fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
        }
    }
    
    /// Navigate to previous search result
    fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            if self.current_search_index == 0 {
                self.current_search_index = self.search_results.len() - 1;
            } else {
                self.current_search_index -= 1;
            }
        }
    }
    
    /// Get current search result for highlighting
    fn get_current_search_result(&self) -> Option<&SearchResult> {
        self.search_results.get(self.current_search_index)
    }
    
    /// Replace current search result
    fn replace_current(&mut self) {
        if let Some(result) = self.get_current_search_result().cloned() {
            let lines: Vec<&str> = self.toml_content.lines().collect();
            if let Some(line) = lines.get(result.line_number - 1) {
                let new_line = if self.search_is_case_sensitive {
                    line.replace(&result.match_text, &self.replace_query)
                } else {
                    // Case-insensitive replace
                    let line_lower = line.to_lowercase();
                    let search_lower = self.search_query.to_lowercase();
                    if let Some(pos) = line_lower.find(&search_lower) {
                        let mut new_line = line.to_string();
                        new_line.replace_range(pos..pos + search_lower.len(), &self.replace_query);
                        new_line
                    } else {
                        line.to_string()
                    }
                };
                
                let mut new_lines = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                new_lines[result.line_number - 1] = new_line;
                self.toml_content = new_lines.join("\n");
                self.has_unsaved_changes = true;
                
                // Re-perform search to update results
                self.perform_search();
            }
        }
    }
    
    /// Replace all occurrences
    fn replace_all(&mut self) {
        if self.search_query.is_empty() || self.search_results.is_empty() {
            return;
        }
        
        if self.search_is_case_sensitive {
            self.toml_content = self.toml_content.replace(&self.search_query, &self.replace_query);
        } else {
            // Case-insensitive replace all
            let lines: Vec<&str> = self.toml_content.lines().collect();
            let mut new_lines = Vec::new();
            
            for line in lines {
                let line_lower = line.to_lowercase();
                let search_lower = self.search_query.to_lowercase();
                
                if line_lower.contains(&search_lower) {
                    let mut new_line = line.to_string();
                    let mut start_pos = 0;
                    while let Some(pos) = new_line[start_pos..].to_lowercase().find(&search_lower) {
                        let actual_pos = start_pos + pos;
                        new_line.replace_range(actual_pos..actual_pos + search_lower.len(), &self.replace_query);
                        start_pos = actual_pos + self.replace_query.len();
                    }
                    new_lines.push(new_line);
                } else {
                    new_lines.push(line.to_string());
                }
            }
            self.toml_content = new_lines.join("\n");
        }
        
        self.has_unsaved_changes = true;
        // Re-perform search to update results
        self.perform_search();
    }
}

impl Default for StyleEditor {
    fn default() -> Self {
        Self::new()
    }
}
