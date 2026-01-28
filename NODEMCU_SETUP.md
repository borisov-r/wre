# NodeMCU ESP32 Setup Guide

Complete installation and setup guide for **NodeMCU ESP32 with ESP32-WROOM-32D module**.

## Table of Contents
- [Hardware Overview](#hardware-overview)
- [Step 1: Install Rust](#step-1-install-rust)
- [Step 2: Install ESP32 Toolchain](#step-2-install-esp32-toolchain)
- [Step 3: Install USB Drivers](#step-3-install-usb-drivers)
- [Step 4: Build and Flash](#step-4-build-and-flash)
- [Troubleshooting](#troubleshooting)

---

## Hardware Overview

### NodeMCU ESP32-WROOM-32D Specifications

- **Chip**: ESP32-D0WD (dual-core Xtensa LX6)
- **Flash**: 4MB
- **RAM**: 520KB SRAM
- **WiFi**: 802.11 b/g/n (2.4GHz only)
- **USB**: CP2102 or CH340 USB-to-Serial chip
- **Voltage**: 3.3V logic, 5V USB power

### Pin Mapping for This Project

```
┌─────────────────────────────────┐
│      NodeMCU ESP32-WROOM-32D    │
├─────────────────────────────────┤
│                                 │
│  3V3  ──────────────────── GND  │
│  EN                         23  │
│  VP(36)                     22  │← DT (Rotary Encoder)
│  VN(39)                     TX  │
│  34                         RX  │
│  35                         21  │← CLK (Rotary Encoder)
│  32   ────────────────────  GND │← Output (LED/Relay)
│  33                         19  │
│  25                         18  │
│  26                          5  │
│  27                         17  │
│  14                         16  │
│  12                          4  │
│  GND                         0  │
│  13                          2  │
│  D2(9)                      15  │
│  D3(10)                      8  │
│  CMD(11)                     7  │
│  5V                          6  │
│                                 │
└─────────────────────────────────┘
```

**Required Connections:**
- **GPIO 21** → Rotary Encoder CLK pin
- **GPIO 22** → Rotary Encoder DT pin
- **GPIO 32** → Output (LED/Relay control)
- **3V3** → Rotary Encoder VCC
- **GND** → Common ground

---

## Step 1: Install Rust

### Linux / macOS

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose option 1 (default installation)

# Reload your shell environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

Expected output:
```
rustc 1.75.0 (or newer)
cargo 1.75.0 (or newer)
```

### Windows

#### Option 1: Using WSL2 (Recommended)

1. **Install WSL2** (Windows Subsystem for Linux):
   ```powershell
   # Open PowerShell as Administrator
   wsl --install
   # Restart your computer
   ```

2. **Install Ubuntu** from Microsoft Store

3. **Follow Linux instructions** inside WSL2 terminal

#### Option 2: Native Windows

1. **Download Rust installer**:
   - Visit https://rustup.rs
   - Download `rustup-init.exe`
   - Run the installer

2. **Install Visual Studio C++ Build Tools**:
   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Select "Desktop development with C++"
   - Install

3. **Verify installation**:
   ```powershell
   rustc --version
   cargo --version
   ```

---

## Step 2: Install ESP32 Toolchain

### All Platforms

```bash
# Install espup (ESP Rust installer)
cargo install espup

# Install ESP32 toolchain (this takes 5-10 minutes)
espup install

# This creates: $HOME/export-esp.sh (Linux/macOS) or 
#              $HOME/export-esp.ps1 (Windows)
```

### Activate ESP Environment

**Linux / macOS:**
```bash
# Temporary (current session only)
source $HOME/export-esp.sh

# Permanent (add to shell profile)
echo 'source $HOME/export-esp.sh' >> ~/.bashrc  # for bash
# OR
echo 'source $HOME/export-esp.sh' >> ~/.zshrc   # for zsh

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

**Windows PowerShell:**
```powershell
# Temporary
. $HOME/export-esp.ps1

# Permanent (add to PowerShell profile)
notepad $PROFILE
# Add this line: . $HOME/export-esp.ps1
```

### Install Additional Tools

```bash
# Install ldproxy (linker proxy for ESP32)
cargo install ldproxy

# Install espflash (flashing tool)
cargo install espflash

# Install cargo-espflash (optional, cargo integration)
cargo install cargo-espflash

# Verify installation
ldproxy --version
espflash --version
```

Expected output:
```
ldproxy 0.3.x
espflash 3.x.x
```

---

## Step 3: Install USB Drivers

### Identify Your USB-to-Serial Chip

NodeMCU ESP32 boards typically use one of these chips:
- **CP2102** (most common)
- **CH340/CH341** (cheaper boards)

To identify, check the chip on your board near the USB port.

### Linux

**CP2102 Driver:**
```bash
# Usually already included in Linux kernel
# Verify with:
lsusb | grep -i "Silicon Labs\|CP210"

# If needed, install:
sudo apt-get update
sudo apt-get install linux-modules-extra-$(uname -r)
```

**CH340 Driver:**
```bash
# Usually already included
# Verify with:
lsusb | grep -i "CH340\|Qinheng"

# Driver loads automatically
```

**Set USB Permissions:**
```bash
# Add your user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in, or:
newgrp dialout

# Create udev rule (optional, for better stability)
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="10c4", ATTR{idProduct}=="ea60", MODE="0666"' | sudo tee /etc/udev/rules.d/99-esp32.rules
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="1a86", ATTR{idProduct}=="7523", MODE="0666"' | sudo tee -a /etc/udev/rules.d/99-esp32.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### macOS

**CP2102 Driver:**
```bash
# Download from Silicon Labs:
# https://www.silabs.com/developers/usb-to-uart-bridge-vcp-drivers

# Or install via Homebrew:
brew tap mengbo/ch340g-ch34g-ch34x-mac-os-x-driver
brew install ch340g-ch34g-ch34x-mac-os-x-driver
```

**CH340 Driver:**
```bash
# Download from:
# https://github.com/adrianmihalko/ch340g-ch34g-ch34x-mac-os-x-driver/releases

# Or install via Homebrew (same as above)
```

After installation, restart your Mac.

### Windows

**CP2102 Driver:**
1. Download from: https://www.silabs.com/developers/usb-to-uart-bridge-vcp-drivers
2. Run the installer
3. Restart your computer

**CH340 Driver:**
1. Download from: http://www.wch.cn/downloads/CH341SER_EXE.html
2. Run the installer
3. Restart your computer

**Verify Driver Installation (Windows):**
1. Connect NodeMCU via USB
2. Open Device Manager
3. Look under "Ports (COM & LPT)"
4. You should see "USB-SERIAL CH340 (COMx)" or "Silicon Labs CP210x USB to UART Bridge (COMx)"
5. Note the COM port number (e.g., COM3)

---

## Step 4: Build and Flash

### 1. Clone Repository

```bash
git clone https://github.com/borisov-r/wre.git
cd wre
```

### 2. Connect NodeMCU ESP32

1. Connect NodeMCU to your computer via USB cable
2. Verify connection:

**Linux/macOS:**
```bash
# List serial ports
ls /dev/tty* | grep -i usb

# You should see something like:
# /dev/ttyUSB0  (Linux)
# /dev/cu.usbserial-*  (macOS)
```

**Windows:**
```powershell
# Check Device Manager for COM port
# Or use espflash to detect:
espflash board-info
```

### 3. Configure WiFi Credentials

```bash
# Set WiFi credentials as environment variables
export WIFI_SSID="YourNetworkName"
export WIFI_PASS="YourNetworkPassword"

# Or create .env file
cat > .env << EOF
WIFI_SSID=YourNetworkName
WIFI_PASS=YourNetworkPassword
EOF

# Then source it
source .env
```

**Windows PowerShell:**
```powershell
$env:WIFI_SSID = "YourNetworkName"
$env:WIFI_PASS = "YourNetworkPassword"
```

### 4. Build the Project

```bash
# First build takes 3-5 minutes
cargo build --release

# Subsequent builds take 30-60 seconds
```

Expected output:
```
   Compiling esp-idf-sys v0.34.0
   Compiling esp-idf-hal v0.43.0
   Compiling wre v0.1.0
    Finished release [optimized] target(s) in 4m 32s
```

### 5. Flash to NodeMCU

**Automatic (espflash auto-detects port):**
```bash
cargo run --release
```

**Manual (specify port):**

**Linux:**
```bash
espflash flash --monitor /dev/ttyUSB0 target/xtensa-esp32-espidf/release/wre
```

**macOS:**
```bash
espflash flash --monitor /dev/cu.usbserial-* target/xtensa-esp32-espidf/release/wre
```

**Windows:**
```powershell
espflash flash --monitor COM3 target/xtensa-esp32-espidf/release/wre
```

### 6. Monitor Serial Output

After flashing, you'll see:
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

**Note the IP address!**

### 7. Access Web Interface

Open your browser to the IP address shown in the serial monitor:
```
http://192.168.1.XXX
```

You should see the Wireless Rotary Encoder web interface!

---

## Troubleshooting

### Build Errors

#### Error: "ldproxy not found"
```bash
cargo install ldproxy
```

#### Error: "esp-idf not found" or "ESP_IDF_VERSION not set"
```bash
# Re-install ESP toolchain
espup install

# Re-source environment
source $HOME/export-esp.sh  # Linux/macOS
. $HOME/export-esp.ps1      # Windows
```

#### Error: "WIFI_SSID environment variable must be set"
```bash
# Make sure to export variables BEFORE building
export WIFI_SSID="YourNetwork"
export WIFI_PASS="YourPassword"
cargo build --release
```

#### Error: Compilation fails with "undefined reference"
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

#### Error: Type mismatch errors (*const u8 vs *const i8) in esp-idf-svc
If you see errors like:
```
error[E0308]: mismatched types
   --> esp-idf-svc-X.XX.X/src/tls.rs:212:36
    |
212 |  rcfg.alpn_protos = bufs.alpn_protos.as_mut_ptr();
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `*mut *const u8`, found `*mut *const i8`
```

This indicates a version compatibility issue between esp-idf-svc and ESP-IDF. The fix is to ensure you're using compatible versions:

```bash
# The Cargo.toml in this project already has the correct versions
# For ESP-IDF v5.1.2, you need:
# - esp-idf-svc = "0.49" or newer (not 0.48)
# - esp-idf-hal = "0.44" or newer
# - esp-idf-sys = "0.34" (not 0.35)
# - embedded-svc = "0.28" or newer

# If you still see this error, clean and rebuild:
cargo clean
cargo build --release
```

**Note:** This project's Cargo.toml has been updated with the correct compatible versions for ESP-IDF v5.1.2. If you cloned an older version of the repository, make sure to pull the latest changes.

### Flash Errors

#### Error: "Permission denied: /dev/ttyUSB0" (Linux)
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in, or:
newgrp dialout

# Try flashing again
```

#### Error: "Failed to connect to ESP32"
**Try these steps in order:**

1. **Press and hold BOOT button** during "Connecting..." message
2. **Lower baud rate:**
   ```bash
   espflash flash --baud 115200 --monitor /dev/ttyUSB0 target/xtensa-esp32-espidf/release/wre
   ```
3. **Reset the board:**
   - Press and release EN (RST) button
   - Try flashing immediately after
4. **Check USB cable:**
   - Some cables are power-only (no data)
   - Try a different cable
5. **Try different USB port:**
   - USB 2.0 ports work better than USB 3.0 sometimes

#### Error: "Timed out waiting for packet header" (Windows)
```powershell
# Use different flash mode
espflash flash --flash-mode dio --baud 115200 --monitor COM3 target/xtensa-esp32-espidf/release/wre
```

#### Error: "Serial port not found"
```bash
# Linux: Check if device is detected
dmesg | tail -20
# Look for "cp210x" or "ch341" messages

# macOS: List all serial devices
ls /dev/cu.*

# Windows: Check Device Manager
# Look under "Ports (COM & LPT)"
```

### Runtime Errors

#### WiFi Not Connecting

1. **Check SSID and password:**
   ```bash
   echo $WIFI_SSID
   echo $WIFI_PASS
   ```
   Make sure they're correct!

2. **Use 2.4GHz network:**
   - ESP32 doesn't support 5GHz WiFi
   - Ensure your router broadcasts 2.4GHz

3. **Check WiFi security:**
   - WPA2-PSK works best
   - Avoid WEP or enterprise networks

4. **Move closer to router:**
   - Poor signal can cause connection issues

#### Web Interface Not Accessible

1. **Verify IP address:**
   - Check serial monitor for "WiFi connected! IP: X.X.X.X"
   
2. **Same network:**
   - Ensure your computer is on the same WiFi network

3. **Firewall:**
   - Check if firewall is blocking port 80

4. **Try ping:**
   ```bash
   ping 192.168.1.XXX
   ```

#### Encoder Not Responding

1. **Check connections:**
   ```
   NodeMCU GPIO 21 ←→ Rotary CLK
   NodeMCU GPIO 22 ←→ Rotary DT
   NodeMCU 3V3     ←→ Rotary VCC
   NodeMCU GND     ←→ Rotary GND
   ```

2. **Verify encoder works:**
   - Measure voltage on CLK/DT pins
   - Should be 3.3V when not rotating

3. **Check serial monitor:**
   - Look for "Target reached" messages
   - If missing, encoder might not be working

#### Output Not Working

1. **Check GPIO 32:**
   - Use multimeter to measure voltage
   - Should be 0V (OFF) or 3.3V (ON)

2. **LED/Relay connection:**
   - Ensure correct polarity
   - May need current-limiting resistor for LED (220Ω)

### Board Not Recognized (Windows)

1. **Install drivers manually:**
   - Right-click on unknown device
   - Select "Update driver"
   - Browse to driver location

2. **Try different USB port:**
   - Some ports have better compatibility

3. **Check cable:**
   - Use cable that came with board
   - Or verified data cable

### ESP32 Won't Boot After Flash

1. **Power cycle:**
   - Unplug USB
   - Wait 5 seconds
   - Plug back in

2. **Erase flash completely:**
   ```bash
   espflash erase-flash /dev/ttyUSB0  # Linux/macOS
   espflash erase-flash COM3          # Windows
   
   # Then flash again
   cargo run --release
   ```

3. **Check power supply:**
   - USB port must provide enough current (500mA+)
   - Try powered USB hub

---

## Additional Resources

### Documentation
- [Quick Start Guide](QUICKSTART.md) - Fast setup
- [Deployment Guide](DEPLOYMENT.md) - Production deployment
- [Testing Guide](TESTING.md) - Testing procedures
- [Architecture](ARCHITECTURE.md) - System design

### Hardware
- [ESP32-WROOM-32D Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf)
- [NodeMCU ESP32 Pinout](https://components101.com/development-boards/nodemcu-esp32-pinout-features-and-datasheet)

### Software
- [ESP-RS Book](https://esp-rs.github.io/book/) - ESP32 Rust programming
- [espflash Documentation](https://github.com/esp-rs/espflash)
- [ESP-IDF Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/)

---

## Quick Reference Card

### Essential Commands
```bash
# Activate ESP environment
source $HOME/export-esp.sh

# Set WiFi credentials
export WIFI_SSID="YourNetwork"
export WIFI_PASS="YourPassword"

# Build
cargo build --release

# Flash (auto-detect port)
cargo run --release

# Flash (specific port)
espflash flash --monitor /dev/ttyUSB0 target/xtensa-esp32-espidf/release/wre

# Monitor only (after flashing)
espflash monitor /dev/ttyUSB0

# Erase flash
espflash erase-flash /dev/ttyUSB0
```

### Common Port Names
- **Linux**: `/dev/ttyUSB0`, `/dev/ttyUSB1`
- **macOS**: `/dev/cu.usbserial-*`, `/dev/cu.SLAB_USBtoUART`
- **Windows**: `COM3`, `COM4`, `COM5`

### GPIO Mapping
- **GPIO 21**: Rotary Encoder CLK
- **GPIO 22**: Rotary Encoder DT
- **GPIO 32**: Output Control
- **3V3**: Power (3.3V)
- **GND**: Ground

---

## Success Checklist

Before you're done, verify:

- [ ] Rust installed (`rustc --version` works)
- [ ] ESP toolchain installed (`espup` completed)
- [ ] ESP environment sourced (`source $HOME/export-esp.sh`)
- [ ] Tools installed (`espflash --version` works)
- [ ] USB driver installed (device appears in `/dev/` or Device Manager)
- [ ] NodeMCU connected (port detected)
- [ ] WiFi credentials set (`echo $WIFI_SSID` shows your network)
- [ ] Project built successfully (`cargo build --release`)
- [ ] Firmware flashed to ESP32
- [ ] Serial monitor shows WiFi connected and IP address
- [ ] Web interface accessible in browser
- [ ] Rotary encoder connected and responding

**If all checked, you're ready to use the Wireless Rotary Encoder! 🎉**

---

## Getting Help

If you encounter issues not covered here:

1. **Check serial monitor output** - Most errors are explained there
2. **Review [TESTING.md](TESTING.md)** - Debugging procedures
3. **GitHub Issues** - Search or create new issue
4. **ESP-RS Community** - https://matrix.to/#/#esp-rs:matrix.org

Good luck with your NodeMCU ESP32 project!
