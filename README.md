# wre
Wireless Rotary Encoder

[![CI](https://github.com/borisov-r/wre/actions/workflows/ci.yml/badge.svg)](https://github.com/borisov-r/wre/actions/workflows/ci.yml)

## 🚀 Quick Start

> **💡 Looking for pre-built firmware?** Check the [Releases](../../releases) page for downloadable binaries!
> 
> **📦 Automatic Releases:** Every successful build on main/master automatically creates a new date-based release.
> 
> **📥 How to Flash:** See the [Flashing Guide](FLASHING.md) for complete instructions on flashing pre-built firmware.

**Using NodeMCU ESP32?** Follow the [NodeMCU ESP32-WROOM-32D Setup Guide](NODEMCU_SETUP.md) for detailed instructions!

**Already familiar with ESP32?** Check out the [Quick Start Guide](QUICKSTART.md) to get running in 15 minutes!

## Overview
This project implements a wireless rotary encoder control system for ESP32, rewritten in Rust with dual-core architecture for optimal performance.

## Features
- **Dual-Core Architecture**: 
  - Core 0: HTTP server with REST API for real-time updates
  - Core 1: Dedicated rotary encoder processing with interrupt handling
- **Web Interface**: Beautiful, responsive UI to control and monitor the encoder
- **Real-time Updates**: Status polling (200ms intervals) to track encoder position and output state
- **Configurable Targets**: Set multiple target angles dynamically
- **Output Control**: GPIO 32 output toggles when target angles are reached
- **Thread-Safe**: Uses Rust's Arc and atomic types for safe cross-core communication

## Hardware Setup

### Supported Boards
- **NodeMCU ESP32 with ESP32-WROOM-32D** (see [detailed setup guide](NODEMCU_SETUP.md))
- Any ESP32 development board with at least 4MB flash

### Connections
- **Rotary Encoder:**
  - CLK pin → GPIO 21
  - DT pin → GPIO 22
  - VCC → 3.3V
  - GND → Ground
  - Both pins use internal pull-up resistors
- **Output:** GPIO 32 (can drive LED, relay, etc.)

## Software Requirements
1. **Rust Toolchain** with ESP32 support:
   ```bash
   # Install Rust if not already installed
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install ESP Rust toolchain
   cargo install espup
   espup install
   source $HOME/export-esp.sh
   
   # Install additional tools
   cargo install ldproxy
   cargo install espflash
   ```

2. **ESP-IDF** (automatically handled by esp-idf-sys)

## Configuration
1. Copy `cfg.toml` to `cfg.toml.local` (ignored by git)
2. Edit `cfg.toml.local` and set your WiFi credentials:
   ```toml
   [wre]
   wifi_ssid = "your_wifi_ssid"
   wifi_password = "your_wifi_password"
   ```

3. Or set environment variables:
   ```bash
   export WIFI_SSID="your_wifi_ssid"
   export WIFI_PASS="your_wifi_password"
   ```

## Building and Flashing
```bash
# Build the project
cargo build --release

# Flash to ESP32 (automatically detects port)
cargo run --release

# Or flash and monitor
espflash flash --monitor target/xtensa-esp32-espidf/release/wre
```

**Note:** If you encounter build errors about type mismatches (`*const i8` vs `*const u8`), make sure you have the latest version from the repository with compatible dependency versions. See [NODEMCU_SETUP.md](NODEMCU_SETUP.md) for troubleshooting.

## Usage
1. Flash the firmware to your ESP32
2. The device will connect to WiFi and display its IP address in the serial monitor
3. Open the IP address in your web browser
4. Use the web interface to:
   - Set target angles (e.g., "45, 90, 135, 180")
   - Start the encoder sequence
   - Monitor real-time position and output status
   - Stop the encoder at any time

## Architecture Details

### Core 0 (HTTP Server)
- Handles WiFi connection
- Runs HTTP server with REST API
- Serves web interface
- Polls encoder state for real-time updates
- Endpoints:
  - `GET /` - Web interface
  - `GET /api/status` - Get current status (JSON)
  - `POST /api/set` - Set target angles (JSON body: `{"angles": [45, 90, 135]}`)
  - `POST /api/stop` - Stop encoder

### Core 1 (Rotary Encoder)
- Handles GPIO interrupts for encoder pins
- Processes encoder state machine (half-step mode for 0.5° resolution)
- Manages output pin control
- Implements bounded range (0-720 half-steps = 0-360°)
- Auto-resets when encoder returns below 2°
- Advances through target angles sequentially

### Cross-Core Communication
- Uses atomic operations and Arc<Mutex<>> for thread-safe state sharing
- Encoder state is accessible from both cores
- Lock-free atomic types (AtomicBool, AtomicI32) for frequently accessed data

## Documentation

### Getting Started
- **[Flashing Guide](FLASHING.md)** - 📥 Complete guide to flashing firmware (pre-built or custom)
- **[NodeMCU ESP32 Setup Guide](NODEMCU_SETUP.md)** - 📱 Complete setup for NodeMCU ESP32-WROOM-32D (Rust installation, drivers, flashing)
- **[Quick Start Guide](QUICKSTART.md)** - ⚡ Get started in 15 minutes (for experienced users)
- **[Deployment Guide](DEPLOYMENT.md)** - 🚀 Production deployment

### Technical Documentation
- **[Architecture Overview](ARCHITECTURE.md)** - 🏗️ System architecture and data flow diagrams
- **[Rust Implementation Details](RUST_IMPLEMENTATION.md)** - 🦀 In-depth architecture and design
- **[Testing Guide](TESTING.md)** - 🧪 Testing procedures and debugging

### Reference
- **[Original MicroPython Code](src/main.py)** - 📜 Reference implementation
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - 📊 Complete feature summary
- **[Project Summary](PROJECT_SUMMARY.md)** - 📝 Visual overview
- **[Release Process](RELEASE.md)** - 🤖 Automatic release system documentation

## Project Structure

```
wre/
├── src/
│   ├── main.rs          # Main application with dual-core setup
│   ├── rotary.rs        # Rotary encoder state machine and logic
│   ├── webserver.rs     # HTTP server and WiFi management
│   ├── main.py          # Original MicroPython implementation (reference)
│   └── boot.py          # MicroPython boot configuration (reference)
├── html/
│   └── index.html       # Web interface with real-time updates
├── Cargo.toml           # Rust dependencies and configuration
├── .cargo/config.toml   # ESP32 build configuration
├── build.rs             # ESP-IDF build script
├── sdkconfig.defaults   # ESP32 SDK configuration
└── rust-toolchain.toml  # Rust toolchain specification
```

## License

See [LICENSE](LICENSE) file for details.

## Original MicroPython Code
The original MicroPython implementation can be found in the `src/` directory for reference.

## Testing push

