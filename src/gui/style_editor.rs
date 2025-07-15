use crate::styles::StyleManager;
use crate::parsers::stylesheet::{StyleSheet, StyleRule, Color};
use egui::{Ui, Color32, ScrollArea};

/// Style editor panel for customizing map appearance
pub struct StyleEditor {
    selected_rule: Option<usize>,
    color_picker_open: bool,
    current_color: Color32,
}

impl StyleEditor {
    pub fn new() -> Self {
        Self {
            selected_rule: None,
            color_picker_open: false,
            current_color: Color32::BLACK,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, style_manager: &mut StyleManager) {
        ui.heading("Style Editor");
        ui.separator();
        
        if let Some(stylesheet) = style_manager.get_active_stylesheet_mut() {
            self.show_stylesheet_editor(ui, stylesheet);
        } else {
            ui.label("No active stylesheet");
        }
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
}

impl Default for StyleEditor {
    fn default() -> Self {
        Self::new()
    }
}
