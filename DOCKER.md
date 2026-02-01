# Docker Build Guide

This guide explains how to use Docker to build the WRE (Wireless Rotary Encoder) firmware with all dependencies pre-installed in a containerized environment.

## Prerequisites

- Docker installed on your system
  - [Docker Desktop](https://www.docker.com/products/docker-desktop/) for Windows/Mac
  - [Docker Engine](https://docs.docker.com/engine/install/) for Linux

## Quick Start

**⚠️ Important Note:** The first time you run the container, it needs to download the ESP Rust toolchain from GitHub. If you encounter GitHub API rate limit errors, see the [Troubleshooting](#esp-toolchain-setup-fails) section below.

### Using the Build Script (Easiest)

The easiest way to build with Docker is using the provided script:

```bash
# Make the script executable (first time only)
chmod +x docker-build.sh

# Build with WiFi credentials
./docker-build.sh -s "your_wifi_ssid" -p "your_wifi_password"

# Or use environment variables
WIFI_SSID="your_ssid" WIFI_PASS="your_pass" ./docker-build.sh
```

### Manual Docker Commands

### Build with Default WiFi Credentials

```bash
# Build the Docker image (this takes 5-10 minutes for first time)
docker build -t wre-builder .

# Run the container to build the firmware (mounts current directory)
docker run --rm -v $(pwd):/project wre-builder
```

The firmware binary will be available at `target/xtensa-esp32-espidf/release/wre`

### Build with Custom WiFi Credentials

```bash
# Build the Docker image
docker build -t wre-builder .

# Run the container with your WiFi credentials
docker run --rm \
  -e WIFI_SSID="your_wifi_ssid" \
  -e WIFI_PASS="your_wifi_password" \
  -v $(pwd):/project \
  wre-builder
```

### Using Docker Compose (Alternative)

You can also use docker-compose for easier management:

```bash
# Set WiFi credentials in environment or .env file
export WIFI_SSID="your_wifi_ssid"
export WIFI_PASS="your_wifi_password"

# Build and run
docker-compose up
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
- ESP Rust toolchain tools (espup, ldproxy)
- ESP-IDF build system (installed on first run)
- All required system dependencies

Build the image (this takes 5-10 minutes the first time):
```bash
docker build -t wre-builder .
```

**Note:** The ESP toolchain setup happens when you run the container (not during image build) to avoid GitHub API rate limit issues during the image build phase.

### Building the Firmware

#### Option 1: Using Volume Mount (Recommended)

This approach mounts your local project directory so the build artifacts persist after the container exits:

```bash
docker run --rm -v $(pwd):/project wre-builder
```

On Windows (PowerShell):
```powershell
docker run --rm -v ${PWD}:/project wre-builder
```

On Windows (CMD):
```cmd
docker run --rm -v %cd%:/project wre-builder
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

Set WiFi credentials when running the container:

```bash
docker run --rm \
  -e WIFI_SSID="MyNetwork" \
  -e WIFI_PASS="MyPassword123" \
  -v $(pwd):/project \
  wre-builder
```

## Interactive Development

For development work, you can run an interactive shell in the container:

```bash
# Start an interactive session
docker run -it --rm -v $(pwd):/project wre-builder /bin/bash

# Inside the container, set up the ESP toolchain (first time only)
espup install
source $HOME/export-esp.sh

# Then you can run commands manually:
cargo build --release
cargo test
```

## Troubleshooting

### Build is Slow

The first build takes a long time (15-30 minutes) because:
1. Docker image build downloads and installs Rust and tools (~5-10 minutes)
2. First container run sets up ESP toolchain (~5 minutes)
3. Rust compiles all crates and ESP-IDF components (~10-20 minutes)

Subsequent builds are much faster:
- Docker image: Cached (seconds)
- ESP toolchain: Already set up in container (or use volume mount)
- Rust builds: Incremental compilation (~1-5 minutes)

### ESP Toolchain Setup Fails

If `espup install` fails (usually due to GitHub API rate limits), you have several options:

**Option 1: Wait and Retry**
GitHub API limits typically reset after an hour. Simply wait and try again.

**Option 2: Use a GitHub Personal Access Token**
Create a [GitHub Personal Access Token](https://github.com/settings/tokens) (with no special permissions needed) and use it:

```bash
docker run --rm \
  -e GITHUB_TOKEN=your_token_here \
  -e WIFI_SSID="your_ssid" \
  -e WIFI_PASS="your_pass" \
  -v $(pwd):/project \
  wre-builder
```

**Option 3: Pre-download ESP Toolchain**
Download the ESP toolchain manually and provide it to the container:

```bash
# On your host machine, install espup and download the toolchain
cargo install espup
espup install

# Then run Docker with the toolchain mounted
docker run --rm \
  -v $(pwd):/project \
  -v $HOME/.espressif:/root/.espressif \
  -v $HOME/export-esp.sh:/root/export-esp.sh \
  wre-builder
```

**Option 4: Manual Setup in Interactive Mode**
Run the container interactively and manually set up the toolchain with retry logic:

```bash
docker run -it --rm -v $(pwd):/project wre-builder /bin/bash

# Inside container, retry until it works:
while ! espup install --log-level info; do
  echo "Retrying in 60 seconds..."
  sleep 60
done

# Then build
source $HOME/export-esp.sh
cargo build --release
```

**Option 5: Use Local Build Instead**
If Docker continues to have issues, consider using a [local build](README.md#software-requirements) instead, which doesn't have the same API limitations.

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
1. Make sure credentials are set as environment variables when running the container
2. Check that credentials don't contain special characters that need escaping
3. Verify environment variables are being passed correctly:
   ```bash
   docker run --rm -e WIFI_SSID="test" -e WIFI_PASS="test" -v $(pwd):/project wre-builder env | grep WIFI
   ```

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
