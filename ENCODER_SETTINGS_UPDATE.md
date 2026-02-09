# Encoder Settings and UI Update

This document describes the comprehensive updates made to the wireless rotary encoder system.

## Summary of Changes

All 12 requirements from the problem statement have been implemented:

### ✅ Completed Changes

1. **Reversed Direction** - Angle now increases when rotating encoder in the opposite direction
2. **Removed Toggle Debug Mode Button** - Debug functionality removed from main UI
3. **Removed Current Angle Field** - Removed from status card (replaced with prominent display)
4. **Removed Target Progress** - No longer shown in status card
5. **Added Settings Page** - New dedicated settings page with navigation
6. **Forward Direction Selection** - Choose between Clockwise (CW) or Counter-Clockwise (CCW)
7. **Step Mode Selection** - Choose between Full step (1°) or Half step (0.5°)
8. **Current Angle Display** - Large, prominent display that updates in real-time
9. **Output Pin Configuration** - Select output GPIO pin and default state (Low/High)
10. **Manual Output Control** - Test buttons to manually set output HIGH or LOW
11. **Settings Integration** - All settings are used throughout the application
12. **Persistent Settings** - Settings are saved to NVS flash storage

## File Changes

### New Files
- `html/settings.html` - New settings page (482 lines)

### Modified Files
- `html/index.html` - Simplified main page with prominent angle display
- `src/rotary.rs` - Added Settings struct and configuration support
- `src/webserver.rs` - Added settings API endpoints and NVS storage
- `src/main.rs` - Integrated settings into encoder task logic

## UI Changes

### Main Page (`/`)
**Removed:**
- Toggle Debug Mode button
- Debug section with pin states
- Current Angle from status card
- Target Progress from status card

**Added:**
- Navigation links (Home | Settings)
- Prominent Current Angle display (large, purple background)
- Cleaner, more focused interface

**Kept:**
- Connection status indicator
- Output status indicator
- Encoder status (Active/Stopped)
- Target angles input field
- Start/Stop buttons

### Settings Page (`/settings`)
**New sections:**

1. **Encoder Configuration**
   - Forward Direction: Radio buttons for CW / Counter-CW
   - Step Mode: Radio buttons for Full (1°) / Half (0.5°)

2. **Output Configuration**
   - Output Pin: Number input for GPIO pin selection
   - Default State: Radio buttons for Low / High
   - Manual Output Control: Current state indicator
   - Test Controls: Buttons to set HIGH or LOW

3. **Actions**
   - Save Settings button (saves to NVS flash)
   - Back to Home button

## API Endpoints

### New Endpoints
- `GET /settings` - Serve settings page
- `GET /api/settings` - Get current settings as JSON
- `POST /api/settings` - Save new settings (to memory and NVS)
- `POST /api/output/manual` - Manually control output pin state

### Existing Endpoints
- `GET /` - Main page
- `GET /api/status` - Get encoder status
- `POST /api/set` - Set target angles
- `POST /api/stop` - Stop encoder

## Technical Implementation

### Settings Structure
```rust
pub struct Settings {
    pub forward_direction: ForwardDirection,  // Clockwise | CounterClockwise
    pub step_mode: StepMode,                   // Full | Half
    pub output_pin: u8,                        // GPIO pin number
    pub output_default_state: PinState,        // Low | High
}
```

### NVS Storage
- Settings are stored in the ESP32's Non-Volatile Storage (NVS)
- Namespace: "storage"
- Key: "encoder_cfg"
- Format: JSON serialization
- Loaded automatically on startup
- Saved when user clicks "Save Settings" button

### Direction Logic
The encoder direction is now configurable:
- **Clockwise**: Positive rotation increases angle (new default)
- **Counter-Clockwise**: Positive rotation decreases angle

The logic in `rotary.rs`:
```rust
let adjusted_direction = match forward_direction {
    ForwardDirection::Clockwise => direction,
    ForwardDirection::CounterClockwise => -direction,
};
```

### Step Mode
The step mode affects how angles are calculated:
- **Full Step**: 1 step = 1 degree (0-360 range uses 360 steps)
- **Half Step**: 2 steps = 1 degree (0-360 range uses 720 steps)

### Manual Output Control
When manual output control is active:
- The output pin is controlled directly via API
- Automatic output control is suspended
- Manual control is cleared when encoder resets to 0°
- Works both when encoder is active and stopped

## Usage

### Changing Settings
1. Navigate to Settings page using the navigation link
2. Adjust desired settings:
   - Select forward direction (CW or CCW)
   - Select step mode (Full or Half)
   - Enter output GPIO pin number
   - Select default output state
3. Click "Save Settings" to persist to flash
4. Settings are applied immediately
5. Output pin changes require device restart

### Testing Output Pin
1. Go to Settings page
2. In the "Output Configuration" section
3. Click "Set HIGH" to turn output on
4. Click "Set LOW" to turn output off
5. The current state indicator updates in real-time

### Using the Encoder
1. On the main page, enter target angles (comma-separated)
2. Click Start to begin
3. Rotate the encoder to the target angles
4. The Current Angle display updates in real-time
5. Output pin activates when target is reached
6. Click Stop to end the sequence

## Backwards Compatibility

### Breaking Changes
- Direction is reversed by default (use CounterClockwise setting for old behavior)
- Debug mode API endpoints still exist but are not used in UI

### Non-Breaking Changes
- Default settings match the original behavior (Half step mode, GPIO32, Low default)
- All existing API endpoints remain functional
- Settings are optional (defaults used if not configured)

## Future Enhancements

Potential improvements that could be added:
- Configurable encoder input pins (currently fixed to GPIO21/22)
- Multiple output pins with individual control
- Angle calibration and offset settings
- Target angle presets/profiles
- Export/import settings as JSON file
- Settings reset to defaults button

## Testing Checklist

When testing this implementation:
- [ ] Main page loads and displays current angle
- [ ] Settings page loads with all options
- [ ] Settings can be changed and saved
- [ ] Settings persist after device restart
- [ ] Forward direction setting works correctly
- [ ] Step mode setting works correctly (verify angle calculations)
- [ ] Manual output control works (test HIGH and LOW)
- [ ] Current angle updates in real-time on both pages
- [ ] Navigation between pages works
- [ ] Start/Stop buttons work as expected
- [ ] Output pin activates at target angles
- [ ] Encoder reset logic works correctly

## Notes

- **Output Pin Configuration**: While the output pin can be selected in settings, changing it requires a device restart as GPIO pins are configured at startup in `main.rs`. This is noted in the UI.

- **NVS Storage**: Settings are saved to flash memory and will persist across power cycles. If NVS fails (corrupted, not initialized, etc.), the application will fall back to default settings.

- **Manual Override**: Manual output control is automatically cleared when the encoder sequence completes or resets, ensuring automatic control resumes for the next sequence.

- **Thread Safety**: All settings access is protected by Mutex to ensure thread-safe operation across the dual-core ESP32 architecture.
