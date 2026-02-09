# Debug Features

This document describes the debug features added to help diagnose and troubleshoot encoder behavior issues.

## Overview

The debug features provide real-time visibility into encoder angle values and detailed serial console logging to help diagnose issues like continuous angle rolling or unexpected behavior.

## Features

### 1. Debug Mode Toggle (Settings Page)

A new "Debug Options" section has been added to the Settings page with a checkbox to enable/disable debug mode.

**Location**: Settings page → Debug Options → Enable Debug Mode

**Behavior**:
- When enabled, additional debug messages are logged to the serial console
- Debug messages include encoder movements, angle values, and target information
- Setting is persisted in NVS (non-volatile storage) and survives restarts

### 2. Current Angle Display (Settings Page)

A real-time display of the current encoder angle has been added to the Settings page.

**Location**: Settings page → Debug Options → Current Angle

**Behavior**:
- Updates every 200ms to show the current encoder position
- Displayed in degrees with one decimal place (e.g., "45.0°")
- Useful for debugging encoder drift or unexpected angle changes

### 3. Start Button Debug Logging

When debug mode is enabled, clicking the Start button logs detailed information to the serial console.

**Serial Console Output** (when debug mode is enabled):
```
I (23333) wre::webserver: Setting target angles: [45.0]
I (23333) wre::webserver: 🔍 DEBUG: Start button clicked - Target angles: [45.0], Current angle: 0.0°
I (23336) wre: 🔄 Encoder reset to 0°
```

### 4. Encoder Movement Debug Logging

When debug mode is enabled, each encoder movement is logged to the serial console.

**Serial Console Output** (when debug mode is enabled and encoder is rotating):
```
I (23500) wre: 🔍 DEBUG: Direction=1 Value=2 Angle=1.0°
I (23520) wre: 🔍 DEBUG: Direction=1 Value=4 Angle=2.0°
I (23540) wre: 🔍 DEBUG: Direction=1 Value=6 Angle=3.0°
```

## How to Use

### Enabling Debug Mode

1. Open the web interface in your browser
2. Navigate to the Settings page
3. Scroll to the "Debug Options" section
4. Check the "Enable Debug Mode" checkbox
5. Click "Save Settings"
6. Debug messages will now appear in the serial console

### Monitoring Current Angle

1. Open the web interface in your browser
2. Navigate to the Settings page
3. Scroll to the "Debug Options" section
4. Observe the "Current Angle" value which updates in real-time
5. This value updates every 200ms even when the encoder is stopped

### Viewing Serial Console Output

To view the serial console output:

```bash
# Using espflash (recommended)
espflash monitor

# Using miniterm (alternative)
miniterm /dev/ttyUSB0 115200

# Using screen (alternative)
screen /dev/ttyUSB0 115200
```

## Troubleshooting with Debug Features

### Issue: Angle Never Stops Rolling

1. Enable debug mode in Settings
2. Observe the serial console while the encoder is supposedly "stopped"
3. Look for continuous `DEBUG: Direction=...` messages
4. Check the "Current Angle" display - it should be stable when encoder is not moving
5. If you see continuous movement messages when encoder is stopped, this indicates:
   - Electrical noise on encoder pins
   - Faulty encoder
   - Loose connections
   - Missing/inadequate pull-up resistors

### Issue: Target Not Reached

1. Enable debug mode in Settings
2. Set a target angle and click Start
3. Observe the debug messages showing current angle vs target
4. Check if the angle is increasing/decreasing as expected
5. Verify the "Current Angle" display matches the expected encoder position

### Issue: Unexpected Resets

1. Enable debug mode in Settings
2. Look for "Encoder reset to 0°" messages in the serial console
3. Check if resets occur when angle drops below the minimum threshold
4. Adjust the "Minimum Angle Threshold" setting if needed

## Technical Details

### Settings Structure

The debug mode is stored in the `Settings` struct as `debug_enabled`:

```rust
pub struct Settings {
    // ... other fields ...
    pub debug_enabled: bool,
}
```

### State Synchronization

- The `debug_enabled` field in Settings is synchronized with the atomic `debug_mode` flag
- When settings are loaded from NVS, debug mode is restored
- When settings are saved, the current debug mode state is persisted

### Performance Impact

Debug mode has minimal performance impact:
- Debug messages are only logged when debug mode is enabled
- The current angle display polling (200ms) is independent of debug mode
- Encoder polling still runs at 1000Hz (1ms) regardless of debug mode

## API Endpoints

### Get Settings (includes debug_enabled)
```bash
curl http://<ESP32_IP>/api/settings
```

Response includes:
```json
{
  "forward_direction": "Clockwise",
  "step_mode": "Half",
  "output_pin": 32,
  "output_default_state": "Low",
  "minimum_angle_threshold": 2.5,
  "hold_output_until_threshold": false,
  "debug_enabled": false
}
```

### Set Settings (including debug_enabled)
```bash
curl -X POST http://<ESP32_IP>/api/settings \
  -H "Content-Type: application/json" \
  -d '{
    "forward_direction": "Clockwise",
    "step_mode": "Half",
    "output_pin": 32,
    "output_default_state": "Low",
    "minimum_angle_threshold": 2.5,
    "hold_output_until_threshold": false,
    "debug_enabled": true
  }'
```

## Files Modified

The following files were modified to implement the debug features:

1. `src/rotary.rs` - Added `debug_enabled` field to Settings struct
2. `src/webserver.rs` - Added debug logging when Start button is clicked
3. `html/settings.html` - Added Debug Options section with toggle and angle display
