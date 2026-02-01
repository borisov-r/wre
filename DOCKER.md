# Docker Build Guide

This guide explains how to use Docker to build the WRE (Wireless Rotary Encoder) firmware with all dependencies pre-installed in a containerized environment.

## Prerequisites

- Docker installed on your system
  - [Docker Desktop](https://www.docker.com/products/docker-desktop/) for Windows/Mac
  - [Docker Engine](https://docs.docker.com/engine/install/) for Linux

## Quick Start

### Build with Default WiFi Credentials

```bash
# Build the Docker image
docker build -t wre-builder .

# Run the container to build the firmware
docker run --rm -v $(pwd)/target:/project/target wre-builder
```

The firmware binary will be available at `target/xtensa-esp32-espidf/release/wre`

### Build with Custom WiFi Credentials

```bash
# Build the Docker image with your WiFi credentials
docker build -t wre-builder \
  --build-arg WIFI_SSID="your_wifi_ssid" \
  --build-arg WIFI_PASS="your_wifi_password" \
  .

# Run the container to build the firmware
docker run --rm -v $(pwd)/target:/project/target wre-builder
```

### Extract the Release Binary

After the build completes, the firmware binary is located at:
```
target/xtensa-esp32-espidf/release/wre
```

You can then flash it to your ESP32:
```bash
espflash flash --monitor target/xtensa-esp32-espidf/release/wre
```

## Detailed Usage

### Building the Docker Image

The Docker image contains all the necessary build tools:
- Rust toolchain (stable)
- ESP Rust toolchain (via espup)
- ESP-IDF build system
- All required system dependencies

Build the image (this takes several minutes the first time):
```bash
docker build -t wre-builder .
```

### Building the Firmware

#### Option 1: Using Volume Mount (Recommended)

This approach mounts your local `target/` directory so the build artifacts persist after the container exits:

```bash
docker run --rm -v $(pwd)/target:/project/target wre-builder
```

On Windows (PowerShell):
```powershell
docker run --rm -v ${PWD}/target:/project/target wre-builder
```

On Windows (CMD):
```cmd
docker run --rm -v %cd%/target:/project/target wre-builder
```

#### Option 2: Copying Files Out of Container

If you prefer not to use volume mounts, you can copy the binary out after building:

```bash
# Run the build
docker run --name wre-build wre-builder

# Copy the binary out
docker cp wre-build:/project/target/xtensa-esp32-espidf/release/wre ./wre

# Clean up
docker rm wre-build
```

### Custom WiFi Credentials

There are two ways to set WiFi credentials:

#### At Build Time (Build Args)

Set credentials when building the Docker image:
```bash
docker build -t wre-builder \
  --build-arg WIFI_SSID="MyNetwork" \
  --build-arg WIFI_PASS="MyPassword123" \
  .
```

#### At Run Time (Environment Variables)

Override credentials when running the container:
```bash
docker run --rm \
  -e WIFI_SSID="MyNetwork" \
  -e WIFI_PASS="MyPassword123" \
  -v $(pwd)/target:/project/target \
  wre-builder
```

## Interactive Development

For development work, you can run an interactive shell in the container:

```bash
# Start an interactive session
docker run -it --rm -v $(pwd):/project wre-builder /bin/bash

# Inside the container, you can run commands manually:
source $HOME/export-esp.sh
cargo build --release
cargo test
```

## Troubleshooting

### Build is Slow

The first build takes a long time (15-30 minutes) because:
1. Docker image build downloads and installs all dependencies
2. Rust compiles all crates and ESP-IDF components

Subsequent builds are faster because Docker caches layers and Rust caches compiled dependencies.

### Out of Disk Space

The Docker image and build artifacts can use several GB of disk space:
- Docker image: ~3-4 GB
- Build artifacts: ~2-3 GB

Clean up with:
```bash
# Remove build artifacts
rm -rf target/

# Remove Docker containers
docker container prune

# Remove Docker images
docker image prune -a
```

### Permission Issues (Linux)

On Linux, Docker runs as root by default, so build artifacts may be owned by root:

```bash
# Fix ownership after build
sudo chown -R $USER:$USER target/
```

Or run Docker with your user ID:
```bash
docker run --rm \
  --user $(id -u):$(id -g) \
  -v $(pwd)/target:/project/target \
  wre-builder
```

### WiFi Credentials Not Working

If you get build errors about WiFi credentials:
1. Ensure credentials are set as build args or environment variables
2. Rebuild the image if you used build args
3. Check that credentials don't contain special characters that need escaping

## Comparison with Local Build

### Docker Advantages
- ✅ Reproducible builds
- ✅ No need to install Rust and ESP toolchain on your system
- ✅ Works the same on any platform (Linux, macOS, Windows)
- ✅ Isolated environment prevents conflicts

### Local Build Advantages
- ✅ Faster builds (no containerization overhead)
- ✅ Better IDE integration
- ✅ Easier debugging
- ✅ Direct access to serial ports for flashing

## CI/CD Integration

You can use this Docker setup in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build firmware
        run: |
          docker build -t wre-builder \
            --build-arg WIFI_SSID="${{ secrets.WIFI_SSID }}" \
            --build-arg WIFI_PASS="${{ secrets.WIFI_PASS }}" \
            .
          docker run --rm -v $(pwd)/target:/project/target wre-builder
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: firmware
          path: target/xtensa-esp32-espidf/release/wre
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [ESP Rust Book](https://esp-rs.github.io/book/)
- [Project README](README.md)
- [Release Guide](RELEASE.md)
