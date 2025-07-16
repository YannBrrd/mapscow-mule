#!/usr/bin/env python3
import xml.etree.ElementTree as ET
import sys

def analyze_svg_for_road_names(svg_file):
    """Analyze an SVG file to check if it contains road names."""
    try:
        tree = ET.parse(svg_file)
        root = tree.getroot()
        
        # Count different element types
        text_elements = root.findall('.//{http://www.w3.org/2000/svg}text')
        path_elements = root.findall('.//{http://www.w3.org/2000/svg}path')
        group_elements = root.findall('.//{http://www.w3.org/2000/svg}g')
        
        print(f"SVG Analysis for: {svg_file}")
        print(f"- Text elements: {len(text_elements)}")
        print(f"- Path elements: {len(path_elements)}")
        print(f"- Group elements: {len(group_elements)}")
        
        # Look for text that might be road names
        road_name_count = 0
        for text in text_elements:
            text_content = text.text
            if text_content and text_content.strip():
                print(f"- Found text: '{text_content.strip()}'")
                # Check if this looks like a road name (not title/metadata)
                if not text_content.startswith("Generated with"):
                    road_name_count += 1
        
        print(f"- Potential road names: {road_name_count}")
        
        # Check for roads group
        roads_group = root.find('.//{http://www.w3.org/2000/svg}g[@id="roads"]')
        if roads_group is not None:
            print("- Found roads group ✓")
            road_texts = roads_group.findall('.//{http://www.w3.org/2000/svg}text')
            print(f"- Text elements in roads group: {len(road_texts)}")
        else:
            print("- No roads group found")
        
        return road_name_count > 0
        
    except Exception as e:
        print(f"Error analyzing SVG: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) > 1:
        svg_file = sys.argv[1]
        has_road_names = analyze_svg_for_road_names(svg_file)
        print(f"\nResult: SVG {'contains' if has_road_names else 'does not contain'} road names")
    else:
        print("Usage: python analyze_svg.py <svg_file>")
