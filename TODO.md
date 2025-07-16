# Mapscow Mule - TODO List

## 🎯 Core Features Completed ✅
- ✅ OSM file loading and parsing
- ✅ Basic map rendering (ways, nodes, buildings)
- ✅ Pan and zoom functionality
- ✅ Outlier coordinate filtering
- ✅ Rectangle zoom selection
- ✅ Style management system
- ✅ Tool panel with navigation tools

## 🎨 UI Improvements Completed ✅
- ✅ **Style Editor Modal**: Converted sidebar style pane to modal window accessible via menu
- ✅ **Map Info to Status Bar**: Moved map information from sidebar to status bar at bottom
- ✅ **Horizontal Toolbar**: Converted vertical tool panel to horizontal toolbar under menu bar
- ✅ **Simplified Navigation Tools**: Removed zoom tool, kept only Pan and Rectangle Zoom
- ✅ **Enhanced Rectangle Zoom UX**: Added crosshair cursor when rectangle zoom mode is active

## 🎨 UI Improvements Pending ⏳
- [ ] **Improve Pan Icon**: Find better icon for Pan tool that aligns with grab cursor functionality
  - Current: 🔀 (twisted arrows) - represents movement but may be confusing
  - Consider: ✋ (hand), 🤏 (pinching hand), ↔️ (left-right arrow), or custom icon
  - Goal: Visual consistency between toolbar icon and mouse cursor behavior

- [x] **POI Display Implementation**: Points of Interest are now shown on the map
  - ✅ Implemented POI rendering in map_view.rs
  - ✅ Parse POI nodes from OSM data (shops, restaurants, amenities, etc.)
  - ✅ Render POIs as colored circles on the map
  - ✅ Add POI filtering by category/type through layer toggle
  - ✅ Implement POI labels on hover/high zoom
  - ✅ Style POIs according to TOML configuration
  - ✅ Added comprehensive POI styles to google-maps.toml
  - ✅ Performance optimization with visibility culling

- [ ] **POI Icon Mapping System**: Implement OpenStreetMap-style icon mapping for POIs
  - Goal: Use standardized POI icons similar to OpenStreetMap's visual language
  - Icon Source: OpenStreetMap icon sets available on GitHub (e.g., osm-icons, maki-icons)
  - Implementation needs:
    - Download/integrate OSM-compatible icon set
    - Create mapping from OSM tags (amenity=restaurant, shop=bakery, etc.) to specific icons
    - Support different icon sizes based on zoom level
    - Implement icon caching for performance
    - Add fallback icons for unmapped POI types
    - Consider using SVG icons for scalability
  - References:
    - OpenStreetMap Carto style POI icons
    - Mapnik XML style sheets for POI rendering
    - Consider Maki icons (Mapbox) or similar open icon sets

## 🚀 High Priority (Core Functionality)

### Map Navigation & Interaction
- [ ] **Keyboard shortcuts**
  - [ ] Arrow keys for panning
  - [ ] +/- keys for zooming
  - [ ] R key for rectangle zoom mode
  - [ ] ESC key to cancel selection/reset tool
  - [ ] Space bar to temporarily switch to pan mode

- [ ] **Enhanced zoom features**
  - [ ] Zoom to specific scale levels (1:1000, 1:5000, etc.)
  - [ ] Fit to window button/shortcut
  - [ ] Zoom history (back/forward)
  - [ ] Mouse wheel zoom sensitivity settings

### File Handling
- [ ] **Recent files menu**
  - [ ] Store recently opened files
  - [ ] Quick access from File menu
  - [ ] Clear recent files option

- [ ] **Drag & drop support**
  - [ ] Drop OSM files directly onto the map
  - [ ] Visual feedback during drag

- [ ] **Multiple file formats**
  - [ ] GPX track loading and display
  - [ ] Better error handling for corrupted files
  - [ ] File format validation

### Map Display & Performance
- [ ] **Rendering optimizations**
  - [ ] Level-of-detail (LOD) system for large datasets
  - [ ] Spatial indexing for fast culling
  - [ ] Asynchronous loading for large files
  - [ ] Progressive rendering (load visible area first)

- [ ] **Visual improvements**
  - [ ] Anti-aliasing for smoother lines
  - [ ] Better color schemes
  - [ ] Configurable map backgrounds
  - [ ] Grid/coordinate overlay option

## 🎨 Medium Priority (User Experience)

### UI/UX Enhancements
- [x] **Status bar**
  - [x] Current coordinates under mouse
  - [x] Map viewport information (zoom, scale, center coordinates)
  - [x] Map statistics (nodes, ways count)
  - [ ] Selected object information
  - [ ] File loading progress

- [ ] **UI Layout Improvements**
  - [x] Remove style pane from sidebar and convert to modal window
    - [x] Add "Style Editor" menu entry under View menu to open modal
    - [x] Convert current style editor side panel to floating modal window
    - [x] Ensure modal window is resizable and can be moved
    - [x] Add close button and keyboard shortcut (ESC) to close modal
  - [x] Move tool panel to horizontal toolbar under menu bar
    - [x] Convert vertical tool panel to horizontal toolbar layout
    - [x] Reorganize tool buttons for horizontal space efficiency
    - [x] Move zoom controls, selection tools, and export buttons to toolbar
    - [ ] Ensure toolbar tools remain accessible and well-organized
    - [ ] Consider grouping related tools with separators
  - [ ] Optimize main map view area
    - [ ] Maximize map viewing space by removing side panels
    - [ ] Ensure toolbar doesn't take excessive vertical space
    - [ ] Maintain current tool functionality while improving layout

- [ ] **Settings/Preferences dialog**
  - [ ] Default zoom levels
  - [ ] Color themes
  - [ ] File handling preferences
  - [ ] Keyboard shortcut customization

- [ ] **Toolbar improvements**
  - [ ] Icon buttons instead of text
  - [ ] Tool tooltips with shortcuts
  - [ ] Customizable toolbar layout

### Measurement & Analysis
- [ ] **Distance measurement tool**
  - [ ] Click points to measure distance
  - [ ] Display in multiple units (m, km, miles)
  - [ ] Area measurement for polygons

- [ ] **Object selection and inspection**
  - [ ] Click to select nodes/ways
  - [ ] Properties panel showing tags
  - [ ] Highlight selected objects
  - [ ] Search by ID or tags

- [ ] **Address geolocation bar**
  - [ ] Geocode address and center map on coordinates

### Export & Sharing
- [ ] **Image export**
  - [ ] PNG export with current view
  - [ ] SVG export for vector graphics
  - [ ] PDF export for printing
  - [ ] Custom resolution settings

- [ ] **Data export**
  - [ ] Export visible area as OSM
  - [ ] Export selected objects
  - [ ] GeoJSON export support

## 🔧 Medium Priority (Technical Improvements)

### Code Quality & Architecture
- [ ] **Error handling**
  - [ ] User-friendly error messages
  - [ ] Graceful handling of memory issues
  - [ ] Recovery from parse errors
  - [ ] Error reporting/logging system

- [ ] **Code organization**
  - [ ] Refactor large methods
  - [ ] Better separation of concerns
  - [ ] Add comprehensive documentation
  - [ ] Unit tests for core functionality

- [ ] **Performance monitoring**
  - [ ] FPS counter option
  - [ ] Memory usage display
  - [ ] Render time profiling

### Configuration & Customization
- [ ] **Style system improvements**
  - [ ] Live style editing
  - [ ] Style import/export
  - [ ] Multiple style presets
  - [ ] Style validation

## 🎨 Style System TODOs (Recently Added Features)

### TOML-Based Style Editor Integration
- [ ] **Complete TOML-based style editor** (Currently shows placeholder)
  - [ ] Replace placeholder text with functional TOML editor
  - [ ] Direct editing of TOML style files in GUI
  - [ ] Real-time preview of style changes
  - [ ] Validation of TOML syntax and structure

### StyleManager Integration Improvements  
- [ ] **Fix StyleManager method compatibility**
  - [ ] Integrate new TOML StyleManager with existing GUI components
  - [ ] Remove hardcoded fallback styles in map_view.rs
  - [ ] Implement proper get_road_style() and get_poi_style() methods
  - [ ] Add get_active_stylesheet() method for TOML StyleManager

### Map Rendering with Configurable Styles
- [ ] **Complete map view integration**
  - [ ] Integrate TOML-based styles with map rendering
  - [ ] Use StyleManager for all drawing functions (water, landuse, buildings, etc.)
  - [ ] Remove "TODO: Integrate with new TOML-based StyleManager" comments
  - [ ] Implement proper style application in all draw_* methods

### Style System Architecture
- [ ] **Resolve dual StyleManager architecture**
  - [ ] Decide between styles/mod.rs vs styles/loader.rs StyleManager
  - [ ] Remove unused StyleManager implementation
  - [ ] Update all imports to use consistent StyleManager
  - [ ] Fix method signature mismatches between implementations

### Export System Integration
- [ ] **Complete SVG export with TOML styles**
  - [ ] Verify all SVG export methods use configurable styles
  - [ ] Test style consistency between GUI and export
  - [ ] Add style selection to export options
  - [ ] Implement PNG export with configurable styles

### Style File Management
- [ ] **Enhanced style file operations**
  - [ ] Save current style modifications back to TOML files
  - [ ] Style file validation and error reporting
  - [ ] Style file backup before modifications
  - [ ] Import/export individual style sections
  - [ ] Style file versioning and migration

### User Experience Improvements
- [ ] **Style editor usability**
  - [ ] Color picker improvements for TOML values
  - [ ] Style property search and filtering
  - [ ] Undo/redo for style changes
  - [ ] Style comparison view (before/after)
  - [ ] Style templates and presets

### Performance Optimizations
- [ ] **Style loading performance**
  - [ ] Cache loaded styles to avoid repeated file I/O
  - [ ] Lazy loading of style files
  - [ ] Style hot-reloading when files change on disk
  - [ ] Memory optimization for large style files

### Error Handling & Validation
- [ ] **Robust style error handling**
  - [ ] Better error messages for malformed TOML files
  - [ ] Graceful fallback when style properties are missing
  - [ ] User-friendly style validation feedback
  - [ ] Style compatibility checks for different map data types

### Documentation & Examples
- [ ] **Style system documentation**
  - [ ] Complete TOML style format documentation
  - [ ] Style creation tutorial
  - [ ] Best practices for custom styles
  - [ ] Example style files for different use cases
  - [ ] Migration guide from old to new style system

- [ ] **Plugin architecture**
  - [ ] Basic plugin system design
  - [ ] Custom tool plugins
  - [ ] Custom renderer plugins

## 🌟 Low Priority (Nice to Have)

### Advanced Features
- [ ] **Multi-layer support**
  - [ ] Layer management panel
  - [ ] Show/hide individual layers
  - [ ] Layer transparency controls
  - [ ] Overlay different data sources

- [ ] **Advanced navigation**
  - [ ] Minimap widget
  - [ ] Bookmark locations
  - [ ] Coordinate search (go to lat/lon)
  - [ ] Address geocoding

- [ ] **Data editing** (Future consideration)
  - [ ] Basic node/way editing
  - [ ] Tag editing interface
  - [ ] Save modifications back to OSM

### Integration & Compatibility
- [ ] **External services**
  - [ ] Online tile overlay option
  - [ ] Geocoding service integration
  - [ ] Route planning integration

- [ ] **Cross-platform improvements**
  - [ ] Better Windows integration
  - [ ] macOS support testing
  - [ ] Linux distribution packages

## 🐛 Known Issues to Fix
- [ ] Fix compiler warnings (dead code, unused variables)
- [ ] Improve coordinate validation edge cases
- [ ] Handle very large files (>100MB OSM files)
- [ ] Memory cleanup for long-running sessions
- [ ] Better handling of corrupted XML files

## 🔧 Code-Level TODOs (From Recent Implementation)

### Style Editor Component (src/gui/style_editor.rs)
- [ ] **Replace placeholder implementation**
  - [ ] Remove "TOML-based style editor coming soon..." placeholder
  - [ ] Implement actual TOML editing interface
  - [ ] Add proper error handling for style operations
  - [ ] Fix unused struct fields (selected_rule, color_picker_open, current_color)

### Map View Integration (src/gui/map_view.rs)
- [ ] **Remove hardcoded style fallbacks**
  - [ ] Replace hardcoded colors in get_way_style() method
  - [ ] Replace hardcoded colors in get_node_style() method
  - [ ] Integrate with new TOML-based StyleManager properly
  - [ ] Remove "TODO: Integrate with new TOML-based StyleManager" comments

### Style Loading System (src/styles/loader.rs)
- [ ] **Complete StyleManager implementation**
  - [ ] Test and verify new_with_default() fallback method
  - [ ] Add proper error handling for style file operations
  - [ ] Implement style file watching for hot-reload
  - [ ] Add methods for style validation

### Method Implementation Gaps
- [ ] **Add missing StyleManager methods**
  - [ ] Implement get_active_stylesheet() for TOML StyleManager
  - [ ] Add get_active_stylesheet_mut() for TOML StyleManager
  - [ ] Implement style rule matching for TOML-based styles
  - [ ] Add style property accessor methods

### Architecture Cleanup
- [ ] **Resolve StyleManager duplication**
  - [ ] Choose between styles/mod.rs and styles/loader.rs implementations
  - [ ] Update all imports to use consistent StyleManager
  - [ ] Remove unused StyleManager implementation
  - [ ] Fix method signature compatibility issues

### Export System (src/export/svg_export.rs)
- [ ] **Verify style integration**
  - [ ] Test all style properties are correctly applied in SVG export
  - [ ] Ensure string type compatibility is maintained
  - [ ] Add comprehensive error handling for style operations
  - [ ] Test export with all available style files

### Compiler Warning Cleanup
- [ ] **Fix style-related warnings**
  - [ ] Remove unused imports from style system
  - [ ] Fix unused variables in style-related code
  - [ ] Address dead code warnings in style modules
  - [ ] Clean up deprecated method usage (e.g., ComboBox::from_id_source)

### File I/O Operations
- [ ] **Style file save/load functionality**
  - [ ] Implement save functionality for modified styles
  - [ ] Add proper file locking for concurrent access
  - [ ] Implement style file backup before modifications
  - [ ] Add atomic write operations for style files

## 📚 Documentation & Distribution

### User Documentation
- [ ] **User manual**
  - [ ] Getting started guide
  - [ ] Feature overview with screenshots
  - [ ] Keyboard shortcuts reference
  - [ ] Troubleshooting guide

- [ ] **Sample data**
  - [ ] Include sample OSM files
  - [ ] Tutorial datasets
  - [ ] Style examples

### Developer Documentation
- [ ] **API documentation**
  - [ ] Code documentation with rustdoc
  - [ ] Architecture overview
  - [ ] Plugin development guide

- [ ] **Build & deployment**
  - [ ] Automated builds (GitHub Actions)
  - [ ] Release packaging
  - [ ] Installation instructions
  - [ ] Dependency management

## 🎯 Next Immediate Steps (Recommended Order)

1. **Fix compiler warnings** - Clean up the codebase
2. **Add keyboard shortcuts** - Essential for power users
3. **Implement status bar** - Shows coordinates and object info
4. **Add recent files menu** - Better file management
5. **Implement distance measurement** - Very useful mapping feature
6. **Add settings dialog** - User customization
7. **Improve error handling** - Better user experience
8. **Add image export** - Share map views

## 💡 Future Vision

### Possible Advanced Features
- Real-time collaborative editing
- Integration with OpenStreetMap API
- 3D rendering mode
- Time-based data visualization
- Mobile companion app
- Web-based version

---

**Note:** This TODO is organized by priority. Focus on High Priority items first to create a solid, usable application. Medium Priority items will enhance user experience, and Low Priority items are for future expansion.

**Estimated Timeline:**
- High Priority: 2-3 weeks
- Medium Priority: 1-2 months  
- Low Priority: 3-6 months

Remember to test each feature thoroughly and get user feedback before moving to the next priority level!
