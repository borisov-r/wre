#!/bin/bash
# Docker build script for WRE firmware
# This script simplifies building the WRE firmware using Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
IMAGE_NAME="wre-builder"
WIFI_SSID="${WIFI_SSID:-your_wifi_ssid}"
WIFI_PASS="${WIFI_PASS:-your_wifi_password}"

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build WRE firmware using Docker"
    echo ""
    echo "Options:"
    echo "  -s, --ssid <SSID>        WiFi SSID (or set WIFI_SSID env var)"
    echo "  -p, --pass <PASSWORD>    WiFi password (or set WIFI_PASS env var)"
    echo "  -b, --build-only         Only build Docker image, don't compile firmware"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 -s MyNetwork -p MyPassword"
    echo "  WIFI_SSID=MyNetwork WIFI_PASS=MyPassword $0"
    exit 1
}

# Parse command line arguments
BUILD_ONLY=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--ssid)
            WIFI_SSID="$2"
            shift 2
            ;;
        -p|--pass)
            WIFI_PASS="$2"
            shift 2
            ;;
        -b|--build-only)
            BUILD_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            ;;
    esac
done

echo -e "${GREEN}=== WRE Docker Build ===${NC}"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    echo "Please install Docker from https://www.docker.com/get-started"
    exit 1
fi

# Build Docker image
echo -e "${YELLOW}Building Docker image...${NC}"
if docker build -t "$IMAGE_NAME" .; then
    echo -e "${GREEN}✓ Docker image built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Docker image${NC}"
    exit 1
fi

if [ "$BUILD_ONLY" = true ]; then
    echo ""
    echo -e "${GREEN}Docker image '$IMAGE_NAME' is ready${NC}"
    echo "To build firmware, run:"
    echo "  docker run --rm -e WIFI_SSID='$WIFI_SSID' -e WIFI_PASS='$WIFI_PASS' -v \$(pwd):/project $IMAGE_NAME"
    exit 0
fi

# Build firmware
echo ""
echo -e "${YELLOW}Building firmware...${NC}"
echo "WiFi SSID: $WIFI_SSID"
echo ""

if docker run --rm \
    -e WIFI_SSID="$WIFI_SSID" \
    -e WIFI_PASS="$WIFI_PASS" \
    -v "$(pwd):/project" \
    "$IMAGE_NAME"; then
    
    echo ""
    echo -e "${GREEN}=== Build Complete ===${NC}"
    echo ""
    echo "Firmware binary location:"
    ls -lh target/xtensa-esp32-espidf/release/wre 2>/dev/null || echo -e "${RED}Binary not found at expected location${NC}"
    echo ""
    echo "To flash to ESP32:"
    echo "  espflash flash --monitor target/xtensa-esp32-espidf/release/wre"
else
    echo ""
    echo -e "${RED}=== Build Failed ===${NC}"
    echo ""
    echo "Common issues:"
    echo "  1. GitHub API rate limit - wait an hour and try again, or use a GitHub token"
    echo "  2. WiFi credentials not set - use -s and -p options or set environment variables"
    echo ""
    echo "For more help, see DOCKER.md"
    exit 1
fi
