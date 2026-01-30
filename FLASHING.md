# Flashing Guide

This guide explains how to flash the WRE (Wireless Rotary Encoder) firmware to your ESP32 device.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installing espflash](#installing-espflash)
- [Flashing Pre-built Firmware](#flashing-pre-built-firmware)
- [Flashing Custom Built Firmware](#flashing-custom-built-firmware)
- [Common Flash Options](#common-flash-options)
- [Troubleshooting](#troubleshooting)
- [Advanced Operations](#advanced-operations)

## Prerequisites

Before flashing:

1. **ESP32 development board** (NodeMCU ESP32, DevKitC, etc.)
2. **USB cable** for connecting ESP32 to computer
3. **USB drivers** installed (see [NODEMCU_SETUP.md](NODEMCU_SETUP.md) for driver installation)
4. **espflash tool** installed (instructions below)

## Installing espflash

espflash is the tool used to flash ESP32 firmware. Install it using cargo:

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install espflash
cargo install espflash

# Verify installation
espflash --version
```

## Flashing Pre-built Firmware

Pre-built firmware is available from the [Releases page](https://github.com/borisov-r/wre/releases).

### Step 1: Download Firmware

1. Go to the [Releases page](https://github.com/borisov-r/wre/releases)
2. Download the latest firmware file (e.g., `wre-esp32-v2026.01.29`)

### Step 2: Connect ESP32

1. Connect your ESP32 to your computer via USB
2. The device should appear as `/dev/ttyUSB0` (Linux), `/dev/cu.usbserial-*` (macOS), or `COM*` (Windows)

### Step 3: Flash Firmware

#### Basic Flash Command

The simplest way to flash (auto-detects port):

```bash
espflash flash --monitor wre-esp32-v2026.01.29
```

This will:
- Automatically detect your ESP32's serial port
- Flash the firmware
- Start the serial monitor to view device output

#### Manual Port Selection

If auto-detection fails, specify the port manually:

**Linux:**
```bash
espflash flash --port /dev/ttyUSB0 --monitor wre-esp32-v2026.01.29
```

**macOS:**
```bash
espflash flash --port /dev/cu.usbserial-0001 --monitor wre-esp32-v2026.01.29
```

**Windows:**
```bash
espflash flash --port COM3 --monitor wre-esp32-v2026.01.29
```

### Step 4: Verify Flash

After flashing, you should see output in the serial monitor:

```
🔧 ESP32 Rotary Encoder Control - Rust Edition
Starting dual-core application...
Starting rotary encoder task on Core 1...
Starting web server on Core 0...
Initializing WiFi...
Connecting to WiFi...
```

### Important: WiFi Credentials

**⚠️ Pre-built firmware contains test WiFi credentials:**
- SSID: `test_ssid`
- Password: `test_password`

These are placeholder values from the CI build system. The device will attempt to connect but will likely fail unless you have a network with these exact credentials.

**To use your own WiFi network:**
- You must build the firmware from source with your credentials
- See [QUICKSTART.md](QUICKSTART.md) or [DEPLOYMENT.md](DEPLOYMENT.md) for build instructions

**Pre-built firmware is still useful for:**
- Testing hardware without WiFi connectivity
- Verifying the device boots and runs correctly
- Evaluating the software before full setup

## Flashing Custom Built Firmware

If you've built the firmware from source with your own WiFi credentials:

```bash
# Flash the built firmware
espflash flash --monitor target/xtensa-esp32-espidf/release/wre

# Or use cargo run (builds and flashes in one command)
cargo run --release
```

## Common Flash Options

### Essential Options

| Option | Description | Example |
|--------|-------------|---------|
| `--monitor` | Start serial monitor after flash | `espflash flash --monitor <firmware>` |
| `--port <PORT>` | Specify serial port | `espflash flash --port /dev/ttyUSB0 <firmware>` |
| `--baud <RATE>` | Set baud rate (default: 460800) | `espflash flash --baud 115200 <firmware>` |

### Advanced Options

| Option | Description | Example |
|--------|-------------|---------|
| `--erase-flash` | Erase entire flash before flashing | `espflash flash --erase-flash <firmware>` |
| `--no-stub` | Disable flasher stub (slower but more reliable) | `espflash flash --no-stub <firmware>` |
| `--verify` | Verify flash after writing | `espflash flash --verify <firmware>` |

### Examples

**Flash with lower baud rate (more reliable for some boards):**
```bash
espflash flash --baud 115200 --monitor wre-esp32-v2026.01.29
```

**Erase flash completely before flashing (useful for troubleshooting):**
```bash
espflash flash --erase-flash --monitor wre-esp32-v2026.01.29
```

**Flash without monitor:**
```bash
espflash flash wre-esp32-v2026.01.29
```

**Monitor serial output later:**
```bash
espflash monitor
# Or specify port:
espflash monitor --port /dev/ttyUSB0
```

## Troubleshooting

### Permission Denied Error

**Linux:**
```bash
# Add your user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in, or restart your session
# Then verify with:
groups | grep dialout
```

### Port Not Found

**List available ports:**
```bash
# Linux/macOS
ls -l /dev/tty* | grep -E "USB|ACM"

# Or use espflash to list ports
espflash board-info
```

**Windows:**
- Open Device Manager
- Look under "Ports (COM & LPT)"
- Your ESP32 should appear as "Silicon Labs CP210x USB to UART Bridge" or similar

### Flash Connection Failed

**Try holding BOOT button:**
1. Hold down the BOOT button on your ESP32
2. Start the flash command
3. Keep holding BOOT until "Connecting..." changes to flashing progress
4. Release BOOT button

**Try lower baud rate:**
```bash
espflash flash --baud 115200 --monitor wre-esp32-v2026.01.29
```

**Try disabling flasher stub:**
```bash
espflash flash --no-stub --monitor wre-esp32-v2026.01.29
```

### Boot Failures After Flash

**Symptoms:** Device resets repeatedly, shows `rst:0x10 (RTCWDT_RTC_RESET)` or `flash read err, 1000`

**Solution:** Erase flash completely and reflash:
```bash
espflash erase-flash
espflash flash --monitor wre-esp32-v2026.01.29
```

### WiFi Not Connecting

If you're using pre-built firmware, remember it has test credentials. To use your WiFi:

1. Set your WiFi credentials:
   ```bash
   export WIFI_SSID="YourNetworkName"
   export WIFI_PASS="YourPassword"
   ```

2. Build from source:
   ```bash
   cargo build --release
   ```

3. Flash your custom build:
   ```bash
   cargo run --release
   ```

## Advanced Operations

### Erase Flash Only

To completely erase the ESP32 flash:

```bash
espflash erase-flash

# Or specify port:
espflash erase-flash --port /dev/ttyUSB0
```

### Read Flash Contents

To backup your ESP32 flash:

```bash
# Read entire 4MB flash
espflash read-flash 0x0 0x400000 backup.bin

# Read specific region
espflash read-flash 0x10000 0x100000 app-backup.bin
```

### Write Flash Contents

To restore a flash backup:

```bash
espflash write-flash 0x0 backup.bin
```

### Monitor Only

To monitor serial output without flashing:

```bash
# Auto-detect port
espflash monitor

# Specify port
espflash monitor --port /dev/ttyUSB0

# Exit monitor with Ctrl+C
```

### Board Information

To get information about your ESP32:

```bash
espflash board-info

# Or specify port:
espflash board-info --port /dev/ttyUSB0
```

### Save Partition Table

To backup the partition table:

```bash
espflash save-partition-table backup.csv
```

## Flash Multiple Devices

To flash multiple ESP32 devices:

```bash
#!/bin/bash
# flash_multiple.sh

FIRMWARE="wre-esp32-v2026.01.29"
PORTS=(
    "/dev/ttyUSB0"
    "/dev/ttyUSB1"
    "/dev/ttyUSB2"
)

for port in "${PORTS[@]}"; do
    echo "Flashing $port..."
    espflash flash --port "$port" "$FIRMWARE"
    if [ $? -eq 0 ]; then
        echo "✓ Successfully flashed $port"
    else
        echo "✗ Failed to flash $port"
    fi
    sleep 2
done

echo "Flashing complete!"
```

Make it executable and run:
```bash
chmod +x flash_multiple.sh
./flash_multiple.sh
```

## Getting Help

- **Documentation:** [README.md](README.md), [QUICKSTART.md](QUICKSTART.md), [DEPLOYMENT.md](DEPLOYMENT.md)
- **NodeMCU Setup:** [NODEMCU_SETUP.md](NODEMCU_SETUP.md)
- **espflash Help:** Run `espflash --help` or `espflash flash --help`
- **Issues:** [GitHub Issues](https://github.com/borisov-r/wre/issues)

## Quick Reference

### Most Common Commands

```bash
# Flash release firmware with monitor
espflash flash --monitor wre-esp32-v2026.01.29

# Flash with manual port
espflash flash --port /dev/ttyUSB0 --monitor wre-esp32-v2026.01.29

# Flash custom build
cargo run --release

# Monitor serial output
espflash monitor

# Erase and reflash
espflash erase-flash
espflash flash --monitor wre-esp32-v2026.01.29
```

### Port Examples

- **Linux:** `/dev/ttyUSB0`, `/dev/ttyACM0`
- **macOS:** `/dev/cu.usbserial-0001`, `/dev/cu.SLAB_USBtoUART`
- **Windows:** `COM3`, `COM4`, `COM5`

---

**Ready to flash?** Download the latest firmware from [Releases](https://github.com/borisov-r/wre/releases) and get started! 🚀
