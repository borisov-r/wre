# Deployment Guide

> **💡 Quick Start Option:** Looking for the fastest way to get started? Download pre-built firmware from the [Releases](https://github.com/borisov-r/wre/releases) page and skip directly to [Flashing Pre-built Firmware](#flashing-pre-built-firmware)!

## Prerequisites Checklist

Before deploying the WRE firmware:

- [ ] ESP32 development board
- [ ] USB cable for programming
- [ ] Rotary encoder with CLK and DT pins
- [ ] Optional: Output device (LED, relay, etc.) for GPIO 32
- [ ] Computer with USB port
- [ ] WiFi network (2.4GHz)

## One-Time Setup

### 1. Install Development Tools

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install ESP Rust toolchain
cargo install espup
espup install
source $HOME/export-esp.sh

# Install flashing tools
cargo install ldproxy espflash
```

Add to your shell profile (`~/.bashrc` or `~/.zshrc`):
```bash
source $HOME/export-esp.sh
```

### 2. Clone Repository

```bash
git clone https://github.com/borisov-r/wre.git
cd wre
```

### 3. Hardware Connections

```
Rotary Encoder Connections:
  CLK  → GPIO 21
  DT   → GPIO 22
  VCC  → 3.3V
  GND  → GND

Output Connections:
  Signal → GPIO 32
  GND    → GND
```

## Building and Flashing

### Method 0: Use Pre-built Firmware (Fastest)

If a release is available, you can skip building from source:

1. Go to the [Releases page](https://github.com/borisov-r/wre/releases)
2. Download the latest `wre-esp32-v*.*.*` firmware (e.g., `wre-esp32-v1.0.0`)
3. Flash it directly:
   ```bash
   espflash flash wre-esp32-v1.0.0
   ```

> **Note:** WiFi credentials are configured after flashing. See the [Configuration section](#wifi-configuration-post-flash) below.

### Method 1: Quick Build (Recommended)

```bash
# Set WiFi credentials
export WIFI_SSID="YourNetworkName"
export WIFI_PASS="YourNetworkPassword"

# Build and flash in one command
cargo run --release
```

### Method 2: Build Then Flash

```bash
# Set WiFi credentials
export WIFI_SSID="YourNetworkName"
export WIFI_PASS="YourNetworkPassword"

# Build only
cargo build --release

# Flash separately
espflash flash --monitor target/xtensa-esp32-espidf/release/wre
```

### Method 3: Using Build Script

```bash
# Set WiFi credentials
export WIFI_SSID="YourNetworkName"
export WIFI_PASS="YourNetworkPassword"

# Run build script
./build.sh

# Flash
espflash flash --monitor target/xtensa-esp32-espidf/release/wre
```

## First Time Flash

First build takes 3-5 minutes. Subsequent builds are faster (30-60 seconds).

### Expected Build Output

```
   Compiling esp-idf-sys v0.34.0
   Compiling esp-idf-hal v0.43.0
   Compiling esp-idf-svc v0.48.0
   Compiling wre v0.1.0 (/path/to/wre)
    Finished release [optimized] target(s) in 4m 32s
```

### Expected Flash Output

```
[00:00:00] ########################################  100% Connecting...
[00:00:01] ########################################  100% Erasing...
[00:00:05] ########################################  100% Writing...
[00:00:10] ########################################  100% Flashing...
```

## Verification

### 1. Check Serial Output

After flashing, you should see:

```
🔧 ESP32 Rotary Encoder Control - Rust Edition
Starting dual-core application...
Starting rotary encoder task on Core 1...
Starting web server on Core 0...
Initializing WiFi...
Connecting to WiFi...
WiFi connected! IP: 192.168.1.XXX
Web server started at http://192.168.1.XXX
Rotary encoder task running on Core 1
```

### 2. Access Web Interface

1. Note the IP address from serial output
2. Open browser to `http://192.168.1.XXX`
3. You should see the "Wireless Rotary Encoder" interface

### 3. Test Basic Functionality

1. Enter angles: `45, 90, 135`
2. Click "Start"
3. Rotate encoder to each angle
4. Verify output toggles at each target

## Troubleshooting Deployment

### Build Errors

**Error: "ldproxy not found"**
```bash
cargo install ldproxy
```

**Error: "WIFI_SSID environment variable must be set"**
```bash
export WIFI_SSID="YourNetwork"
export WIFI_PASS="YourPassword"
```

**Error: "esp-idf not found"**
```bash
# Re-run ESP toolchain setup
espup install
source $HOME/export-esp.sh
```

### Flash Errors

**Error: "Permission denied: /dev/ttyUSB0"**
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER
# Log out and back in
```

**Error: "Port not found"**
```bash
# List available ports
ls -l /dev/tty*

# Try different port
espflash flash --port /dev/ttyUSB1 --monitor target/xtensa-esp32-espidf/release/wre
```

**Error: "Failed to connect"**
```bash
# Try lower baud rate
espflash flash --baud 115200 --monitor target/xtensa-esp32-espidf/release/wre

# Press and hold BOOT button during flash
```

### Runtime Errors

**WiFi not connecting**
- Verify SSID and password are correct
- Ensure 2.4GHz network (ESP32 doesn't support 5GHz)
- Check router security settings (WPA2 recommended)
- Move ESP32 closer to router

**Web interface not accessible**
- Verify IP address from serial monitor
- Ensure computer is on same WiFi network
- Try `ping <esp32-ip>` to test connectivity
- Check firewall settings

**Encoder not responding**
- Verify GPIO connections (CLK=21, DT=22)
- Check encoder power (3.3V)
- Test with multimeter for pin continuity
- Try swapping CLK and DT pins

**Output not working**
- Verify GPIO 32 connection
- Check output device (LED, relay)
- Monitor serial logs for "Target reached" messages
- Use multimeter to verify GPIO 32 voltage

## Production Deployment

### Hardening for Production

1. **Disable Debug Logging**
   ```rust
   // In src/main.rs
   log::set_max_level(log::LevelFilter::Info);
   ```

2. **Enable Watchdog Timer**
   ```rust
   // Add to main.rs
   use esp_idf_sys::esp_task_wdt_init;
   unsafe { esp_task_wdt_init(30, true); }
   ```

3. **Add OTA Support** (Future enhancement)
   - Enable OTA partition in sdkconfig
   - Add OTA update endpoint

### Persistent Configuration

Store WiFi credentials in NVS (Future enhancement):
```rust
// Instead of compile-time env variables
// Store in Non-Volatile Storage
```

### Multiple Devices

To deploy multiple devices:

1. Create device-specific .env files:
   ```bash
   # device1.env
   WIFI_SSID=Network1
   WIFI_PASS=Pass1
   
   # device2.env
   WIFI_SSID=Network2
   WIFI_PASS=Pass2
   ```

2. Build for each device:
   ```bash
   source device1.env && cargo build --release
   espflash flash --port /dev/ttyUSB0 target/xtensa-esp32-espidf/release/wre
   
   source device2.env && cargo build --release
   espflash flash --port /dev/ttyUSB1 target/xtensa-esp32-espidf/release/wre
   ```

## Continuous Deployment

### Automated Flashing Script

```bash
#!/bin/bash
# flash_all.sh

DEVICES=(
    "/dev/ttyUSB0"
    "/dev/ttyUSB1"
    "/dev/ttyUSB2"
)

export WIFI_SSID="YourNetwork"
export WIFI_PASS="YourPassword"

cargo build --release

for device in "${DEVICES[@]}"; do
    echo "Flashing $device..."
    espflash flash --port "$device" target/xtensa-esp32-espidf/release/wre
    sleep 2
done
```

### Remote Monitoring

Set up centralized logging:
1. Add MQTT support for remote logs
2. Use syslog for error reporting
3. Add Prometheus metrics endpoint

## Backup and Recovery

### Save Configuration

```bash
# Backup partition table
espflash save-partition-table backup.csv

# Save flash contents
espflash read-flash 0x0 0x400000 backup.bin
```

### Recovery

```bash
# Restore flash
espflash write-flash 0x0 backup.bin

# Or reflash firmware
cargo run --release
```

## Updates and Maintenance

### Firmware Updates

1. Pull latest code:
   ```bash
   git pull origin main
   ```

2. Rebuild and reflash:
   ```bash
   cargo build --release
   espflash flash --monitor target/xtensa-esp32-espidf/release/wre
   ```

### Monitoring

Check device health:
```bash
# Serial monitoring
espflash monitor /dev/ttyUSB0

# Web interface status
curl http://192.168.1.XXX/api/status
```

## Support

- [Quick Start Guide](QUICKSTART.md) - Getting started
- [Testing Guide](TESTING.md) - Testing procedures
- [Architecture](ARCHITECTURE.md) - System design
- [GitHub Issues](https://github.com/borisov-r/wre/issues) - Report problems
