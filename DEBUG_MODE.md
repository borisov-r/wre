# Debug Mode for Rotary Encoder

## Overview

A debug mode has been added to help diagnose issues with the rotary encoder. This feature allows you to see the real-time state of the encoder pins and internal state machine.

## Features

### Debug Information Display

When debug mode is enabled, the following information is displayed:

1. **CLK Pin (GPIO21)**: Current state (HIGH/LOW) of the clock pin
2. **DT Pin (GPIO22)**: Current state (HIGH/LOW) of the data pin
3. **State Machine**: Internal state machine value (hexadecimal)
4. **Raw Value**: Raw encoder value in half-steps (0-720)
5. **Calculated Angle**: The current angle in degrees (0-360°)

### How to Use

1. Open the web interface in your browser
2. Click the "🔍 Toggle Debug Mode" button
3. A yellow debug section will appear showing real-time encoder information
4. Rotate the encoder and observe the pin states and values
5. Click the button again to hide the debug section

## API Endpoints

### Enable/Disable Debug Mode
**POST** `/api/debug`
```json
{
  "enabled": true
}
```

### Get Debug Information
**GET** `/api/debug/info`

Response:
```json
{
  "clk_pin": true,
  "dt_pin": false,
  "state_machine": 3,
  "raw_value": 180,
  "angle": 90.0
}
```

## Understanding the State Machine

The rotary encoder uses a half-step state machine with the following states:

- `0x00` (R_START): Starting state
- `0x01` (R_CW_1): Clockwise transition 1
- `0x02` (R_CW_2): Clockwise transition 2
- `0x03` (R_CW_3): Clockwise transition 3
- `0x04` (R_CCW_1): Counter-clockwise transition 1
- `0x05` (R_CCW_2): Counter-clockwise transition 2

The state machine also includes direction flags:
- `0x10` (DIR_CW): Clockwise direction detected
- `0x20` (DIR_CCW): Counter-clockwise direction detected

## Troubleshooting with Debug Mode

### Issue: Encoder not responding
- Check if CLK and DT pins are changing when you rotate the encoder
- If pins don't change, check physical connections

### Issue: Encoder direction is reversed
- The code has a `reverse` flag (currently set to `true`)
- If rotation direction is wrong, the flag may need to be changed

### Issue: Encoder is too sensitive or not sensitive enough
- Check the state machine transitions
- Verify the encoder is a half-step type (common for KY-040 modules)

### Issue: Random value jumps
- Look for rapid pin state changes in the debug info
- May indicate electrical noise or bounce issues
- Consider adding hardware debouncing capacitors

## Performance Note

Debug mode stores pin states and state machine information on every interrupt, which adds minimal overhead (a few atomic stores per interrupt). This should not affect normal operation.
