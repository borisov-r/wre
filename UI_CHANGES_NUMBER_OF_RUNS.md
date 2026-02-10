# UI Changes Visualization

## Home Page - New Run Progress Display

### Before
```
┌────────────────────────────────────────────────────┐
│  Output Status  │  Encoder Status  │  Current Angle│
│       🔴        │     Stopped      │     0.0°      │
└────────────────────────────────────────────────────┘
```

### After
```
┌────────────────────────────────────────────────────┐
│  Output Status  │  Encoder Status  │  Current Angle│
│       🔴        │     Stopped      │     0.0°      │
│────────────────────────────────────────────────────│
│                Run Progress                         │
│                   0 / 1                             │  ← NEW ROW
└────────────────────────────────────────────────────┘
```

## Settings Page - New Number of Runs Field

### Before
```
┌────────────────────────────────────────────────────┐
│ Encoder Configuration                               │
│                                                     │
│ Number of Target Angles         [▼ 1    ]          │
│ Select how many target angle inputs to show         │
│                                                     │
│ Forward Direction        ◉ Clockwise  ○ Counter-CW │
│ ...                                                 │
└────────────────────────────────────────────────────┘
```

### After
```
┌────────────────────────────────────────────────────┐
│ Encoder Configuration                               │
│                                                     │
│ Number of Target Angles         [▼ 1    ]          │
│ Select how many target angle inputs to show         │
│                                                     │
│ Number of Runs                  [ 1      ]          │  ← NEW FIELD
│ How many times to repeat all target angles          │  ← NEW HELP TEXT
│ (1-100000)                                          │  ← NEW HELP TEXT
│                                                     │
│ Forward Direction        ◉ Clockwise  ○ Counter-CW │
│ ...                                                 │
└────────────────────────────────────────────────────┘
```

## Run Progress States

### State 1: Before Start
```
Run Progress: 0 / 1
```

### State 2: During Run 1 of 5
```
Run Progress: 1 / 5
Encoder Status: Active
Current Angle: 45.0°
```

### State 3: During Run 3 of 5
```
Run Progress: 3 / 5
Encoder Status: Active
Current Angle: 90.0°
```

### State 4: All Runs Complete
```
Run Progress: 5 / 5
Encoder Status: Stopped
Current Angle: 0.0°
```

## Workflow Example

### Setup (Settings Page)
1. User navigates to Settings
2. Sets "Number of Runs" to 5
3. Clicks "Save Settings"
4. Returns to Home

### Execution (Home Page)
1. User enters target angles: 45°, 90°, 135°
2. Presses "Start" button
3. **Run 1**: System rotates through 45° → 90° → 135° → returns to 0°
   - Display shows "Run Progress: 1 / 5"
4. **Run 2**: System rotates through 45° → 90° → 135° → returns to 0°
   - Display shows "Run Progress: 2 / 5"
5. **Runs 3-5**: Process repeats
6. **Completion**: After Run 5, encoder automatically stops
   - Display shows "Run Progress: 5 / 5"
   - Encoder Status shows "Stopped"

## API Response Examples

### Before (Original)
```json
{
  "active": true,
  "angle": 45.0,
  "target_angles": [45.0, 90.0, 135.0],
  "current_target_index": 0,
  "output_on": false,
  "target_reached": false
}
```

### After (With Run Progress)
```json
{
  "active": true,
  "angle": 45.0,
  "target_angles": [45.0, 90.0, 135.0],
  "current_target_index": 0,
  "output_on": false,
  "target_reached": false,
  "current_run": 1,
  "total_runs": 5
}
```
