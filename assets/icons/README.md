# Mapscow Mule Icon Setup Instructions

## Step 1: Convert SVG to PNG

You have a few options to convert the SVG to PNG:

### Option A: Using Inkscape (Recommended)
1. Install Inkscape from https://inkscape.org/
2. Open Command Prompt and run:
```bash
inkscape assets/icons/mapscow-mule.svg --export-type=png --export-filename=assets/icons/mapscow-mule.png --export-width=512 --export-height=512
```

### Option B: Using online converter
1. Go to https://cloudconvert.com/svg-to-png
2. Upload the `mapscow-mule.svg` file
3. Set dimensions to 512x512 pixels
4. Download and save as `mapscow-mule.png` in the icons folder

### Option C: Using GIMP
1. Open GIMP
2. File → Open → Select the SVG file
3. Set import size to 512x512 pixels
4. File → Export As → Save as PNG

## Step 2: Include PNG in the application

The code will automatically include the PNG file at compile time and use it as the application icon.

## Icon Design Features

The created icon includes:
- Blue circular background representing maps/geography
- Map view with roads (orange highways, yellow streets)
- Buildings in gray
- Water features in blue
- Rectangle selection overlay (representing your zoom feature)
- Magnifying glass icon
- "MAPSCOW MULE" text at the bottom

Feel free to modify the SVG file to customize colors, elements, or text before converting to PNG!
