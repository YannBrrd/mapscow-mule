# Logging Configuration

This project now uses proper logging instead of `println!` statements for debug output.

## Setup

The project uses:
- `log` crate for structured logging
- `env_logger` crate for configurable log output

## Usage

### Running with logging

Set the `RUST_LOG` environment variable to control log levels:

```powershell
# Debug level (shows all debug, info, warn, error messages)
$env:RUST_LOG="debug"
cargo run

# Info level (shows info, warn, error messages)
$env:RUST_LOG="info"
cargo run

# Warn level (shows only warn and error messages)
$env:RUST_LOG="warn"
cargo run

# Error level (shows only error messages)
$env:RUST_LOG="error"
cargo run
```

### Log Levels Used

- **`debug!`**: Detailed debugging information for map rendering, road processing, coordinate calculations
- **`info!`**: Important application events like successful startup, completed operations
- **`warn!`**: Warnings about data issues like invalid coordinates or suspicious data
- **`error!`**: Not yet implemented (would be for serious errors)

### Examples

#### Debug Output
When running with `RUST_LOG="debug"`, you'll see detailed information about:
- Map viewport calculations
- Road rendering statistics
- Coordinate bounds detection
- Specific road debugging (for roads containing "bezons" or "bernanos")
- User interactions like zoom operations

#### Info Output
When running with `RUST_LOG="info"`, you'll see:
- Application startup messages
- Rectangle zoom selection completion
- Icon loading status

#### Module-specific Logging
You can also filter by module:

```powershell
# Only show debug logs from the map_view module
$env:RUST_LOG="mapscow_mule::gui::map_view=debug"
cargo run

# Show debug for OSM parser, info for everything else
$env:RUST_LOG="mapscow_mule::parsers::osm=debug,info"
cargo run
```

## Migration from println!

All `println!` debug statements have been replaced with appropriate log levels:

- `println!("DEBUG: ...")` → `debug!(...)`
- `println!("Warning: ...")` → `warn!(...)`
- User-facing completion messages → `info!(...)`

The logger is initialized in `main.rs` with `env_logger::init()`.
