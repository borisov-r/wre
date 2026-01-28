#!/bin/bash
# Build and flash script for WRE

set -e

# Check if WiFi credentials are set
if [ -z "$WIFI_SSID" ] || [ -z "$WIFI_PASS" ]; then
    echo "Error: WiFi credentials not set!"
    echo "Please set WIFI_SSID and WIFI_PASS environment variables:"
    echo "  export WIFI_SSID='your_ssid'"
    echo "  export WIFI_PASS='your_password'"
    echo ""
    echo "Or create a .env file with these variables."
    exit 1
fi

echo "Building WRE for ESP32..."
echo "WiFi SSID: $WIFI_SSID"

# Build the project
cargo build --release

echo ""
echo "Build complete! To flash:"
echo "  cargo run --release"
echo "or"
echo "  espflash flash --monitor target/xtensa-esp32-espidf/release/wre"
