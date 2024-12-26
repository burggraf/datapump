#!/bin/bash

# Exit on any error
set -e

echo "Cleaning up old icons..."
# Create icons directory if it doesn't exist
mkdir -p src-tauri/icons

# Remove all existing icons and temp directories
rm -rf src-tauri/icons/*
rm -rf ico_temp icon.iconset

echo "Starting icon generation with fresh SVG..."
# Generate standard PNG files
echo "Generating 32x32.png"
rsvg-convert -w 32 -h 32 static/icon.svg -o src-tauri/icons/32x32.png

echo "Generating 128x128.png"
rsvg-convert -w 128 -h 128 static/icon.svg -o src-tauri/icons/128x128.png

echo "Generating 128x128@2x.png"
rsvg-convert -w 256 -h 256 static/icon.svg -o src-tauri/icons/128x128@2x.png

# Generate numbered PNG files
for size in 32 128 256 512 1024; do
  echo "Generating icon${size}.png"
  rsvg-convert -w ${size} -h ${size} static/icon.svg -o src-tauri/icons/icon${size}.png
done

# Also generate app-icon- versions
for size in 32 128 256 512 1024; do
  echo "Generating app-icon-${size}.png"
  rsvg-convert -w ${size} -h ${size} static/icon.svg -o src-tauri/icons/app-icon-${size}.png
done

# Generate ICO file (Windows)
echo "Generating temporary PNGs for ICO"
mkdir -p ico_temp
# Create an array to store the filenames
ico_files=""
for size in 256 128 96 64 48 32 16; do
  echo "Creating ${size}x${size} for ICO"
  rsvg-convert -w ${size} -h ${size} static/icon.svg -o ico_temp/icon_${size}.png
  ico_files="$ico_files ico_temp/icon_${size}.png"
done
echo "Combining PNGs into ICO"
magick convert $ico_files -depth 8 src-tauri/icons/icon.ico
rm -rf ico_temp

# Generate ICNS file (macOS)
echo "Generating ICNS file..."
mkdir -p icon.iconset
echo "16x16"
rsvg-convert -w 16 -h 16 static/icon.svg -o icon.iconset/icon_16x16.png
echo "16x16@2x"
rsvg-convert -w 32 -h 32 static/icon.svg -o icon.iconset/icon_16x16@2x.png
echo "32x32"
rsvg-convert -w 32 -h 32 static/icon.svg -o icon.iconset/icon_32x32.png
echo "32x32@2x"
rsvg-convert -w 64 -h 64 static/icon.svg -o icon.iconset/icon_32x32@2x.png
echo "64x64"
rsvg-convert -w 128 -h 128 static/icon.svg -o icon.iconset/icon_128x128.png
echo "64x64@2x"
rsvg-convert -w 256 -h 256 static/icon.svg -o icon.iconset/icon_128x128@2x.png
echo "128x128"
rsvg-convert -w 256 -h 256 static/icon.svg -o icon.iconset/icon_256x256.png
echo "128x128@2x"
rsvg-convert -w 512 -h 512 static/icon.svg -o icon.iconset/icon_256x256@2x.png
echo "256x256"
rsvg-convert -w 512 -h 512 static/icon.svg -o icon.iconset/icon_512x512.png
echo "256x256@2x"
rsvg-convert -w 1024 -h 1024 static/icon.svg -o icon.iconset/icon_512x512@2x.png
echo "Creating ICNS file"
iconutil -c icns icon.iconset -o src-tauri/icons/icon.icns
echo "Cleaning up"
rm -rf icon.iconset

# Copy original SVG to Tauri icons directory
echo "Copying original SVG"
cp static/icon.svg src-tauri/icons/icon.svg

echo "Icon generation complete!"
