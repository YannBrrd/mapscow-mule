# Style Selection Dropdown Feature

## Overview

A new style selection dropdown has been added to the **View** menu that allows users to dynamically switch between different map styles.

## Location

The style selector can be found in:
**View Menu → Style: [Dropdown]**

## Available Styles

Currently, two styles are included:

1. **Google Maps** - Professional Google Maps-like appearance with warm colors
2. **OSM Default** - Traditional OpenStreetMap styling

## How to Use

1. Open the application 
2. Go to the **View** menu in the top menu bar
3. Look for **Style:** with a dropdown next to it
4. Click the dropdown to see available styles:
   - **Google Maps** 
   - **OSM Default**
5. Select your preferred style
6. The map styling will update immediately
7. Status messages will confirm successful style changes

## Style Files

Styles are stored as TOML configuration files in:
```
assets/styles/
├── google-maps.toml    # Google Maps style
└── osm-default.toml    # OSM Default style
```

## Adding New Styles

To add a new style:

1. Create a new `.toml` file in `assets/styles/`
2. Follow the same structure as existing files
3. The style will automatically appear in the dropdown
4. Style names are automatically formatted (e.g., "my-custom-style" becomes "My Custom Style")

## Technical Details

- Styles are loaded dynamically from TOML files
- No restart required when adding new styles
- The dropdown automatically scans the `assets/styles/` directory
- Selected style is remembered in the GUI state
- Full integration with the export system (SVG exports use the selected style)

## Status Messages

The application provides feedback when switching styles:
- Success: "Loaded [Style Name] style"  
- Error: "Error loading style [Style Name]: [error details]"

## Default Behavior

- Application starts with "Google Maps" style by default
- If Google Maps style fails to load, it falls back to a basic style
- The dropdown shows user-friendly names while maintaining technical file names internally

This feature provides a seamless way to switch between different map appearances without needing to restart the application or modify configuration files manually.
