# Style System Documentation

## Overview

The style system has been completely refactored to use external TOML configuration files instead of hardcoded values. This provides a flexible, configurable approach to styling maps with proper Google Maps-style appearance.

## Architecture

### 1. Style Files Location
All style configurations are located in: `assets/styles/`

- `google-maps.toml` - Google Maps style configuration
- `osm-default.toml` - Default OSM style configuration

### 2. Core Components

#### `src/styles/loader.rs`
- **MapStyle**: Main structure containing all styling properties
- **StyleManager**: Manages loading and accessing different styles
- TOML deserialization for complex nested configurations

#### `src/utils/config.rs`
- **AppConfig**: Updated to include `default_style` preference
- Defaults to "google-maps" style
- Configurable via preferences file

#### `src/export/svg_export.rs`
- **SvgExporter**: Updated to use StyleManager instead of hardcoded values
- All color and styling values now come from TOML configuration
- Proper string type handling for SVG library compatibility

## Style Configuration Structure

### TOML Format
```toml
[background]
color = "#F2EFE9"

[water]
color = "#A5BFDD"
border_color = "#9BB3D1"
border_width = 1.0

[roads]
highway_primary_color = "#F4C430"
highway_primary_width = 8.0
highway_primary_border_color = "#D4A017"
highway_primary_border_width = 1.0
# ... more road types

[landuse]
forest_color = "#C8E6C9"
grass_color = "#E8F5E8"
# ... more landuse types

[pois]
restaurant_color = "#FF6B35"
shop_color = "#4CAF50"
# ... more POI types

[buildings]
color = "#E0E0E0"
border_color = "#BDBDBD"
border_width = 0.5

[boundaries]
administrative_color = "#9E9E9E"
administrative_width = 1.0
administrative_dash = "2,2"

[labels]
font_family = "Arial"
road_label_stroke = "#FFFFFF"
road_label_stroke_width = 2.0
poi_label_stroke = "#FFFFFF"
poi_label_stroke_width = 1.5
place_label_stroke = "#FFFFFF"
place_label_stroke_width = 2.0
```

## Google Maps Style Features

The Google Maps style (`google-maps.toml`) implements:

1. **Color Scheme**: 
   - Warm beige background (#F2EFE9)
   - Blue water (#A5BFDD)
   - Appropriate contrast for readability

2. **Road Hierarchy**:
   - Primary highways: Golden yellow with darker borders
   - Secondary roads: White/light colors
   - Residential: Light gray
   - All with proper width scaling

3. **Points of Interest**:
   - Restaurants: Orange (#FF6B35)
   - Shops: Green (#4CAF50)
   - Tourism: Purple (#9C27B0)
   - Hotels: Blue (#2196F3)

4. **Landuse**:
   - Forests: Light green (#C8E6C9)
   - Grass: Very light green (#E8F5E8)
   - Buildings: Light gray (#E0E0E0)

5. **Labels**:
   - White stroke outlines for readability
   - Arial font family
   - Appropriate stroke widths

## Usage

### Default Configuration
The application automatically loads the style specified in `default_style` preference (defaults to "google-maps").

### Adding New Styles
1. Create a new `.toml` file in `assets/styles/`
2. Follow the same structure as existing files
3. Update the `default_style` preference to use the new style

### Style Properties Access
The StyleManager provides methods to access specific styling:
- `get_road_style(road_type)` - Get road styling by type
- `get_poi_style(poi_type)` - Get POI styling by type
- Direct access to all style categories

## Benefits

1. **Externalized Configuration**: No need to recompile for style changes
2. **User Customization**: Users can create their own style files
3. **Professional Appearance**: Google Maps-like styling
4. **Maintainable**: Clear separation of styling from logic
5. **Extensible**: Easy to add new style categories and properties

## Technical Implementation

- Uses `toml = "0.8"` for TOML parsing
- Serde deserialization for type safety
- Result-based error handling for missing files/properties
- String reference handling compatible with SVG library requirements
- Integrated with existing export system

The style system is now ready for production use with professional Google Maps-style appearance and full configurability.
