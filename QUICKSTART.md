# Quick Start Guide

Get your Wireless Rotary Encoder running in 15 minutes!

> **📱 Using NodeMCU ESP32?** For more detailed instructions including USB drivers and troubleshooting, see the [NodeMCU ESP32-WROOM-32D Setup Guide](NODEMCU_SETUP.md).

## Prerequisites

- ESP32 development board (NodeMCU ESP32, DevKitC, etc.)
- Rotary encoder with CLK and DT pins
- USB cable for programming
- Linux, macOS, or WSL on Windows

## Step 1: Install Rust and ESP Tools (5 minutes)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install ESP Rust toolchain
cargo install espup
espup install

# Source the ESP environment (add this to your .bashrc/.zshrc)
source $HOME/export-esp.sh

# Install flashing tools
cargo install ldproxy espflash
```

## Step 2: Hardware Setup (2 minutes)

Connect your rotary encoder to ESP32:

```
Rotary Encoder    →    ESP32
─────────────────────────────
CLK               →    GPIO 21
DT                →    GPIO 22
+                 →    3.3V
GND               →    GND

Output (LED/Relay)
─────────────────────────────
Control           →    GPIO 32
GND               →    GND
```

## Step 3: Configure WiFi (1 minute)

```bash
# Clone the repository
git clone https://github.com/borisov-r/wre.git
cd wre

# Set WiFi credentials
export WIFI_SSID="YourWiFiNetwork"
export WIFI_PASS="YourWiFiPassword"

# Or create a .env file
cat > .env << EOF
WIFI_SSID=YourWiFiNetwork
WIFI_PASS=YourWiFiPassword
EOF
```

## Step 4: Build and Flash (5 minutes)

```bash
# First build takes 3-5 minutes
cargo build --release

# Flash to ESP32 (auto-detects port)
cargo run --release

# Or manually specify port
espflash flash --monitor /dev/ttyUSB0 target/xtensa-esp32-espidf/release/wre
```

## Step 5: Use the Web Interface (2 minutes)

1. Watch the serial monitor for the IP address:
   ```
   WiFi connected! IP: 192.168.1.100
   Web server started at http://192.168.1.100
   ```

2. Open the IP address in your web browser

3. Enter target angles (e.g., "45, 90, 135, 180")

4. Click "Start" and rotate your encoder!

5. Watch the output pin trigger when you reach each angle

## Testing Without Hardware

You can still explore the web interface without physical hardware:

1. Build and flash as above
2. The encoder will read all 0s (no movement)
3. Web interface will still work
4. Output control will function
5. Use this to understand the UI before wiring up

## Troubleshooting

### "ldproxy not found"
```bash
cargo install ldproxy
```

### "Permission denied /dev/ttyUSB0"
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### WiFi not connecting
- Check SSID and password
- Ensure 2.4GHz network (ESP32 doesn't support 5GHz)
- Check serial monitor for error messages

### Web interface not loading
- Verify IP address from serial monitor
- Check firewall settings
- Ensure ESP32 and computer are on same network

### Encoder not responding
- Verify GPIO connections (CLK=21, DT=22)
- Check encoder power supply
- Look for interrupt messages in serial monitor

## Next Steps

- Read [RUST_IMPLEMENTATION.md](RUST_IMPLEMENTATION.md) for architecture details
- Customize target angles for your application
- Add your own output control logic
- Integrate with home automation systems

## Need Help?

- Check the [README.md](README.md) for detailed documentation
- Review serial monitor output for errors
- Verify hardware connections
- Ensure ESP32 is properly powered

Happy encoding! 🎛️
