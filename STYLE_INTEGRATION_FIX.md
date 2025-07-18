# Style System Integration Fix

## Problem Summary

The application had **two separate style systems** that were not properly integrated:

1. **SVG Export**: Used the new TOML-based `StyleManager` from `src/styles/loader.rs`
2. **GUI Onscreen Display**: Used hardcoded color values in `src/gui/map_view.rs`

This caused styles to apply only to SVG exports but not to the onscreen map display.

## Root Cause Analysis

### SVG Export (Working Correctly)
- Used `StyleManager` to load styles from TOML files in `assets/styles/`
- Called methods like `style.get_road_style(highway)` for dynamic styling
- Properly integrated with the style dropdown system

### GUI Map View (Using Hardcoded Styles)
- Used hardcoded methods like `get_road_casing_style()` and `get_road_fill_style()`
- Had fixed color values like `Color32::from_rgb(231, 170, 56)` in the code
- Ignored the selected style from the dropdown

## Solution Implemented

### 1. Road Casing Rendering
**Before:**
```rust
let (casing_width, casing_color) = self.get_road_casing_style(highway);
```

**After:**
```rust
let (casing_width, casing_color) = {
    let style = style_manager.get_current_style();
    let (_, _, border_color, border_width) = style.get_road_style(highway);
    
    if border_width > 0.0 && !border_color.is_empty() {
        let (r, g, b) = Self::hex_to_rgb(border_color);
        (border_width, Color32::from_rgb(r, g, b))
    } else {
        (0.0, Color32::TRANSPARENT)
    }
};
```

### 2. Water Areas Rendering
**Before:**
```rust
painter.add(egui::Shape::convex_polygon(
    points,
    Color32::from_rgb(170, 218, 255), // Hardcoded light blue
    egui::Stroke::new(1.0, Color32::from_rgb(110, 180, 240)), // Hardcoded border
));
```

**After:**
```rust
let style = style_manager.get_current_style();
let water_color = Self::hex_to_rgb(&style.water.color);
let fill_color = Color32::from_rgba_unmultiplied(
    water_color.0, 
    water_color.1, 
    water_color.2, 
    (255.0 * style.water.opacity) as u8
);

painter.add(egui::Shape::convex_polygon(
    points,
    fill_color,
    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(
        water_color.0.saturating_sub(30), 
        water_color.1.saturating_sub(30), 
        water_color.2.saturating_sub(30), 
        255
    )),
));
```

### 3. Landuse Areas Rendering
**Before:**
```rust
match landuse.as_str() {
    "forest" | "wood" => {
        fill_color = Color32::from_rgb(200, 220, 188); // Hardcoded
        stroke_color = Color32::from_rgb(180, 200, 168); // Hardcoded
        should_draw = true;
    },
    // ... more hardcoded colors
}
```

**After:**
```rust
// Use StyleManager for landuse colors
if let Some(landuse) = way.tags.get("landuse") {
    if let Some(color_str) = style.get_landuse_color(landuse) {
        let (r, g, b) = Self::hex_to_rgb(color_str);
        fill_color = Color32::from_rgb(r, g, b);
        should_draw = true;
    }
} else if let Some(leisure) = way.tags.get("leisure") {
    if let Some(color_str) = style.get_leisure_color(leisure) {
        let (r, g, b) = Self::hex_to_rgb(color_str);
        fill_color = Color32::from_rgb(r, g, b);
        should_draw = true;
    }
} else if let Some(natural) = way.tags.get("natural") {
    if let Some(color_str) = style.get_natural_color(natural) {
        let (r, g, b) = Self::hex_to_rgb(color_str);
        fill_color = Color32::from_rgb(r, g, b);
        should_draw = true;
    }
}
```

### 4. Railway Rendering
**Before:**
```rust
painter.add(egui::Shape::dashed_line(
    &points,
    egui::Stroke::new(2.0, Color32::from_rgb(120, 120, 120)), // Hardcoded
    10.0,
    5.0,
));
```

**After:**
```rust
let style = style_manager.get_current_style();
let rail_color = Self::hex_to_rgb(&style.railway.rail_color);

painter.add(egui::Shape::dashed_line(
    &points,
    egui::Stroke::new(style.railway.rail_width, Color32::from_rgb(rail_color.0, rail_color.1, rail_color.2)),
    10.0,
    5.0,
));
```

## Result

Now both the onscreen display and SVG export use the **same StyleManager system**:

✅ **Style dropdown changes** now apply to **both** onscreen display and SVG export  
✅ **TOML style files** control **all** rendering  
✅ **Consistent styling** between preview and export  
✅ **Easy customization** through configuration files  

## Files Modified

- `src/gui/map_view.rs` - Updated rendering methods to use StyleManager
  - `draw_road_casings()` - Now uses StyleManager for road border styling
  - `draw_water_areas()` - Now uses StyleManager for water colors and opacity
  - `draw_landuse_areas()` - Completely rewritten to use StyleManager
  - `draw_railways()` - Now uses StyleManager for railway styling

## Testing

The application builds successfully with only warnings (no errors). You can now:

1. Change the style dropdown in the menu
2. See the changes immediately reflected in the onscreen map view
3. Export to SVG and see the same styling applied

The hardcoded style functions are marked as legacy and deprecated, ensuring backward compatibility while encouraging use of the new unified system.
