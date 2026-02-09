# Settings Page UI Changes

## New Debug Options Section

The Settings page now includes a new "Debug Options" section below the "Output Configuration" section.

## Visual Layout

```
┌─────────────────────────────────────────────────────────────┐
│  ⚙️ Settings                                                 │
│  Configure Encoder Parameters                                │
│                                                               │
│  [ Home ]  [ Settings ]                                       │
│                                                               │
│  ✅ Connected                                                 │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│  Encoder Configuration                                        │
│                                                               │
│  Forward Direction           ◉ Clockwise  ○ Counter-CW       │
│  Step Mode                   ◉ Full (1°)  ○ Half (0.5°)      │
│  Minimum Angle Threshold (°)              [2.5]              │
│  Angle below which the encoder is considered to be at 0°     │
├─────────────────────────────────────────────────────────────┤
│  Output Configuration                                         │
│                                                               │
│  Output Pin (GPIO)                        [32]               │
│  Default State               ◉ Low  ○ High                    │
│  Hold Output Until Threshold ☑                               │
│  Keep output HIGH until angle drops below minimum threshold  │
│                                                               │
│  Manual Output Control       Current: ⚫                      │
│                                                               │
│  [ Set HIGH ]  [ Set LOW ]                                    │
│  Use manual control for testing the output pin               │
├─────────────────────────────────────────────────────────────┤
│  *** NEW *** Debug Options                                   │
│                                                               │
│  Enable Debug Mode           ☑                               │
│  Show debug messages in serial console including angle       │
│  values and encoder movements                                │
│                                                               │
│  Current Angle               45.0°                           │
│  Real-time encoder angle for debugging purposes              │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│         [ 💾 Save Settings ]                                  │
│         [ ← Back to Home ]                                    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Key Visual Elements

### Debug Options Section
- **Background**: Light gray (`#f8f9fa`) with rounded corners
- **Border**: 2px solid light gray (`#e9ecef`)
- **Padding**: 20px
- **Section Title**: "Debug Options" in bold, 16px font

### Enable Debug Mode Checkbox
- **Type**: Checkbox input (20x20 px)
- **Label**: "Enable Debug Mode"
- **Position**: Left-aligned label, right-aligned checkbox
- **Help text**: Gray, small font (12px) below the checkbox

### Current Angle Display
- **Type**: Read-only text display
- **Value**: Dynamically updated every 200ms
- **Format**: "XX.X°" (e.g., "45.0°")
- **Color**: Purple/blue (`#667eea`)
- **Font**: Bold, 16px
- **Position**: Right-aligned in the setting row
- **Help text**: Gray, small font (12px) below the display

## Interaction Behavior

### Debug Mode Checkbox
1. User clicks checkbox to enable/disable debug mode
2. State is saved when "Save Settings" button is clicked
3. Setting is persisted in flash memory (NVS)
4. When enabled, debug messages appear in serial console

### Current Angle Display
1. Automatically updates every 200ms
2. Shows current encoder position even when stopped
3. Updates continue whether encoder is active or stopped
4. No user interaction required

## Color Scheme

The new section maintains consistency with existing sections:
- **Section background**: `#f8f9fa`
- **Section border**: `#e9ecef`
- **Text labels**: `#666`
- **Status values**: `#333` (normal text), `#667eea` (highlighted values)
- **Help text**: `#6c757d`
- **Button gradient**: `#667eea` to `#764ba2`

## Responsive Design

The section maintains responsive behavior:
- Checkbox and labels stack on narrow screens
- Current angle display remains readable on all screen sizes
- Help text wraps appropriately
- All elements scale with parent container

## Serial Console Output Examples

### When Debug Mode is Enabled

#### Starting the Encoder:
```
I (23333) wre::webserver: Setting target angles: [45.0]
I (23333) wre::webserver: 🔍 DEBUG: Start button clicked - Target angles: [45.0], Current angle: 0.0°
I (23336) wre: 🔄 Encoder reset to 0°
```

#### During Encoder Rotation:
```
I (23500) wre: 🔍 DEBUG: Direction=1 Value=2 Angle=1.0°
I (23520) wre: 🔍 DEBUG: Direction=1 Value=4 Angle=2.0°
I (23540) wre: 🔍 DEBUG: Direction=1 Value=6 Angle=3.0°
I (23560) wre: 🔍 DEBUG: Direction=1 Value=8 Angle=4.0°
...
I (25500) wre: 🔍 DEBUG: Direction=1 Value=90 Angle=45.0°
I (25520) wre: ⚡ Target reached: 45.0°
```

#### When Target is Reached:
```
I (25520) wre: ⚡ Target reached: 45.0°
I (25700) wre: 🔍 DEBUG: Direction=-1 Value=88 Angle=44.0°
I (25900) wre: 🔍 DEBUG: Direction=-1 Value=86 Angle=43.0°
...
```

#### Returning to Zero:
```
I (30000) wre: 🔍 DEBUG: Direction=-1 Value=4 Angle=2.0°
I (30100) wre: 🔍 DEBUG: Direction=-1 Value=2 Angle=1.0°
I (30200) wre: 🔍 DEBUG: Direction=-1 Value=0 Angle=0.0°
I (30300) wre: 🔄 Encoder reset to 0°
I (30301) wre: ✅ All targets completed and returned to 0°.
```

### When Debug Mode is Disabled

Only standard operational messages are shown:
```
I (23333) wre::webserver: Setting target angles: [45.0]
I (23336) wre: 🔄 Encoder reset to 0°
I (25520) wre: ⚡ Target reached: 45.0°
I (30300) wre: 🔄 Encoder reset to 0°
I (30301) wre: ✅ All targets completed and returned to 0°.
```

## Browser Compatibility

The UI changes are compatible with:
- ✅ Chrome/Chromium (desktop & mobile)
- ✅ Firefox (desktop & mobile)
- ✅ Safari (desktop & iOS)
- ✅ Edge (desktop)
- ✅ Opera (desktop & mobile)

All features use standard HTML5, CSS3, and JavaScript ES6 without framework dependencies.
