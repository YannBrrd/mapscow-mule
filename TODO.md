# Mapscow Mule - TODO List

## 🎯 Core Features Completed ✅
- ✅ OSM file loading and parsing
- ✅ Basic map rendering (ways, nodes, buildings)
- ✅ Pan and zoom functionality
- ✅ Outlier coordinate filtering
- ✅ Rectangle zoom selection
- ✅ Style management system
- ✅ Tool panel with navigation tools

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
- [ ] **Status bar**
  - [ ] Current coordinates under mouse
  - [ ] Selected object information
  - [ ] File loading progress
  - [ ] Map statistics (nodes, ways count)

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
