#!/bin/bash

# Exit on error
set -e

# Load environment variables from .env file
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "No .env file found. Please create one with APPLE_ID and APPLE_ID_PASSWORD variables."
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting build process for datapump...${NC}"

# Step 1: Install dependencies
echo -e "${YELLOW}Installing dependencies...${NC}"
npm install
(cd src-tauri && cargo check)

# Step 2: Build the app
echo -e "${YELLOW}Building the application...${NC}"
npm run build
npm run tauri:build

# Step 3: Sign and Notarize the app
echo -e "${YELLOW}Preparing to sign and notarize the application...${NC}"

# Get the path to the app bundle
APP_PATH="src-tauri/target/release/bundle/macos/datapump.app"

# Check if we have a valid signing identity in the login keychain
if ! security find-identity -v -p codesigning login.keychain-db | grep -q "Developer ID Application"; then
    echo -e "${RED}No valid Developer ID Application certificate found in login keychain!${NC}"
    echo -e "${RED}Please ensure you have a valid Apple Developer ID certificate installed in your login keychain.${NC}"
    exit 1
fi

# Get the first valid Developer ID Application certificate from login keychain
SIGNING_IDENTITY=$(security find-identity -v -p codesigning login.keychain-db | grep "Developer ID Application" | head -n 1 | cut -d '"' -f 2)

echo -e "${YELLOW}Signing with identity: $SIGNING_IDENTITY${NC}"

# Sign the app using the login keychain with hardened runtime
echo -e "${YELLOW}Signing the application...${NC}"
codesign --force --deep --options runtime --keychain ~/Library/Keychains/login.keychain-db --sign "$SIGNING_IDENTITY" "$APP_PATH"

# Create a ZIP archive for notarization
echo -e "${YELLOW}Creating ZIP archive for notarization...${NC}"
ditto -c -k --keepParent "$APP_PATH" "datapump.zip"

# Check if environment variables are set
if [ -z "$APPLE_ID" ] || [ -z "$APPLE_ID_PASSWORD" ] || [ -z "$APPLE_TEAM_ID" ]; then
    echo -e "${RED}APPLE_ID, APPLE_ID_PASSWORD, or APPLE_TEAM_ID not set in .env file${NC}"
    exit 1
fi

# Notarize the app
echo -e "${YELLOW}Submitting app for notarization...${NC}"
xcrun notarytool submit "datapump.zip" --apple-id "$APPLE_ID" --password "$APPLE_ID_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait

# Clean up ZIP file
rm "datapump.zip"

# Staple the notarization ticket
echo -e "${YELLOW}Stapling notarization ticket to app...${NC}"
xcrun stapler staple "$APP_PATH"

# Verify the signature and notarization
echo -e "${YELLOW}Verifying signature and notarization...${NC}"
codesign --verify --deep --strict "$APP_PATH"
spctl --assess --type execute --verbose "$APP_PATH"

# Create a DMG (optional)
echo -e "${YELLOW}Creating DMG...${NC}"
if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "datapump" \
        --volicon "src-tauri/icons/icon.icns" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "datapump.app" 175 190 \
        --hide-extension "datapump.app" \
        --app-drop-link 425 190 \
        "target/datapump.dmg" \
        "$APP_PATH"
else
    echo "create-dmg not found. Skipping DMG creation."
    echo "To install create-dmg: brew install create-dmg"
fi

echo -e "${GREEN}Build, signing, and notarization process completed!${NC}"
echo -e "${GREEN}Your app is ready at: $APP_PATH${NC}"
if [ -f "target/datapump.dmg" ]; then
    echo -e "${GREEN}DMG file created at: target/datapump.dmg${NC}"
fi
