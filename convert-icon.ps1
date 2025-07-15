# PowerShell script to convert SVG to PNG using Inkscape
# Make sure Inkscape is installed and in your PATH

$svgPath = "assets\icons\mapscow-mule.svg"
$pngPath = "assets\icons\mapscow-mule.png"

Write-Host "Converting SVG to PNG..."
Write-Host "Input: $svgPath"
Write-Host "Output: $pngPath"

# Check if SVG file exists
if (-not (Test-Path $svgPath)) {
    Write-Error "SVG file not found: $svgPath"
    exit 1
}

# Try to convert using Inkscape
try {
    & inkscape $svgPath --export-type=png --export-filename=$pngPath --export-width=512 --export-height=512
    
    if (Test-Path $pngPath) {
        Write-Host "✓ Successfully converted SVG to PNG!" -ForegroundColor Green
        Write-Host "Icon created: $pngPath" -ForegroundColor Green
        
        # Show file size
        $fileSize = (Get-Item $pngPath).Length
        Write-Host "File size: $($fileSize / 1KB) KB" -ForegroundColor Green
    } else {
        Write-Error "PNG file was not created"
    }
} catch {
    Write-Error "Failed to convert SVG to PNG. Make sure Inkscape is installed and in your PATH."
    Write-Host "Alternative: Use an online converter like https://cloudconvert.com/svg-to-png" -ForegroundColor Yellow
}

Write-Host "`nNext steps:"
Write-Host "1. Run 'cargo build' to compile with the new icon"
Write-Host "2. The icon will appear in the title bar and taskbar when you run the app"
