@echo off
echo Building Mapscow Mule...

echo.
echo === Checking Rust version ===
rustc --version
cargo --version

echo.
echo === Running cargo check ===
cargo check

if %ERRORLEVEL% neq 0 (
    echo.
    echo ❌ Cargo check failed!
    pause
    exit /b 1
)

echo.
echo === Building in debug mode ===
cargo build

if %ERRORLEVEL% neq 0 (
    echo.
    echo ❌ Debug build failed!
    pause
    exit /b 1
)

echo.
echo ✅ Build successful!
echo.
echo To run the application:
echo   cargo run
echo.
echo To build for release:
echo   cargo build --release
echo.
pause
