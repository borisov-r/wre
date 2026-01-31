# Debug Mode Feature Implementation Summary

## Problem Statement
The rotary encoder on the ESP32 device was not working correctly after uploading the Release version. To diagnose the issue, we needed a way to observe the real-time state of the encoder pins and internal state machine.

## Solution
Added a debug mode feature accessible through a button on the web interface that displays:
- Real-time CLK pin state (GPIO21)
- Real-time DT pin state (GPIO22)
- Internal state machine value
- Raw encoder value (half-steps)
- Calculated angle in degrees

## Implementation Details

### Backend (Rust)
1. **src/rotary.rs**
   - Added atomic fields for debug mode flag and pin/state capture
   - Modified ISR to capture values only when debug mode is enabled (minimal overhead)
   - Proper memory ordering: Release/Acquire for synchronization, Relaxed for debug values
   - Single atomic load per interrupt for optimal performance

2. **src/webserver.rs**
   - POST `/api/debug` - Enable/disable debug mode
   - GET `/api/debug/info` - Retrieve current debug information
   - Generic error messages to prevent information disclosure
   - Stack-allocated buffers for efficiency

### Frontend (HTML/JavaScript)
1. **html/index.html**
   - Yellow debug section with clear visual separation
   - Real-time updates every 200ms when debug mode is active
   - Toggle button to enable/disable debug mode
   - Proper error handling for network failures

### Documentation
1. **DEBUG_MODE.md**
   - Usage instructions
   - API documentation
   - Troubleshooting guide
   - State machine explanation

## Preview
![Debug Mode Screenshot](https://github.com/user-attachments/assets/902f895c-aee8-4dd3-b702-1b40080c221a)

The screenshot shows the debug information section (yellow background) displaying real-time encoder data.

## Code Quality
- ✅ All code review feedback addressed
- ✅ No security vulnerabilities (CodeQL clean)
- ✅ Optimized for ISR performance
- ✅ Proper error handling
- ✅ Memory ordering correctness
- ✅ Information disclosure prevention

## Usage
1. Open the web interface
2. Click "🔍 Toggle Debug Mode" button
3. Rotate the encoder and observe pin states changing in real-time
4. Use the information to diagnose encoder issues:
   - Check if pins are responding to rotation
   - Verify state machine transitions
   - Confirm angle calculations are correct
   - Identify noise or bounce issues

## Performance Impact
- When disabled: Zero overhead (no atomic stores)
- When enabled: Minimal overhead (4 atomic stores per interrupt using Relaxed ordering)
- Debug mode only affects ISR with one additional Acquire load to check the flag

## Troubleshooting Use Cases
1. **Pins not responding** → Check physical connections
2. **Wrong direction** → May need to adjust reverse flag
3. **Erratic values** → May indicate electrical noise/bounce
4. **State machine stuck** → May indicate encoder type mismatch

## Security Considerations
- Generic error messages prevent information disclosure
- Proper JSON serialization prevents injection attacks
- Debug information only reveals encoder state (no sensitive system info)
- No authentication required (device is on local network)
