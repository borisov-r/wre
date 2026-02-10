# Number of Runs Feature

## Overview
The Number of Runs feature allows users to configure the system to automatically repeat all target angles for a specified number of runs (1 to 100,000).

## User Interface Changes

### Settings Page (`/settings`)
- **New Field**: "Number of Runs" input field
  - Range: 1 to 100,000
  - Default: 1
  - Location: Encoder Configuration section, after "Number of Target Angles"
  - Description: "How many times to repeat all target angles (1-100000)"

### Home Page (`/`)
- **New Display**: "Run Progress" status indicator
  - Shows: "Current Run / Total Runs" (e.g., "3 / 10")
  - Location: Status card, below the Current Angle display
  - Updates in real-time as runs complete

## How It Works

1. **Configuration**: User sets the Number of Runs in the Settings page
2. **Execution**: When Start button is pressed:
   - Run counter initializes to 1
   - System processes all target angles in sequence
   - When encoder returns to 0° after the last target angle, the run counter increments
   - Process repeats until all runs are completed
3. **Completion**: After all runs are finished, the encoder automatically stops

## Technical Implementation

### Backend Changes

#### `src/rotary.rs`
- Added `number_of_runs: u32` field to `Settings` struct (default: 1)
- Added `current_run: Arc<AtomicI32>` to `RotaryEncoderState`
- Added `total_runs: Arc<AtomicI32>` to `RotaryEncoderState`
- Added helper methods:
  - `get_current_run()`: Returns current run number
  - `get_total_runs()`: Returns total number of runs configured
  - `set_total_runs(runs: i32)`: Sets total runs
  - `increment_current_run()`: Increments current run counter
  - `reset_current_run()`: Resets current run to 0
- Modified `set_target_angles()` to initialize run counters from settings

#### `src/webserver.rs`
- Updated `StatusResponse` struct to include:
  - `current_run: i32`
  - `total_runs: i32`
- API `/api/status` now returns run progress information

#### `src/main.rs`
- Modified rotary task logic to handle multiple runs:
  - When all targets complete, checks if more runs remain
  - If yes: increments run counter and resets target index to 0
  - If no: stops the encoder

### Frontend Changes

#### `html/settings.html`
- Added Number of Runs input field in Encoder Configuration section
- Added JavaScript to load and save the `number_of_runs` setting

#### `html/index.html`
- Added "Run Progress" display in status card
- Added JavaScript to update run counter from API status

## Example Usage

### Single Run (Default)
1. Set Number of Runs to 1 in Settings
2. Set target angles (e.g., 45°, 90°, 135°)
3. Press Start
4. Display shows "Run Progress: 1 / 1"
5. System processes angles and stops

### Multiple Runs
1. Set Number of Runs to 5 in Settings
2. Set target angles (e.g., 45°, 90°, 135°)
3. Press Start
4. Display shows "Run Progress: 1 / 5"
5. System processes angles, returns to 0°
6. Display shows "Run Progress: 2 / 5"
7. Repeats until "Run Progress: 5 / 5"
8. System stops after final run completes

## API Documentation

### GET `/api/status`
Response now includes:
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

### GET/POST `/api/settings`
Settings now include:
```json
{
  "forward_direction": "Clockwise",
  "step_mode": "Half",
  "output_pin": 32,
  "output_default_state": "Low",
  "minimum_angle_threshold": 2.5,
  "hold_output_until_threshold": false,
  "debug_enabled": false,
  "num_target_angles": 1,
  "tick_size_multiplier": 2.0,
  "number_of_runs": 1
}
```

## Notes
- The run logic maintains all existing behavior for each individual run
- Each run follows the same sequence: reach targets → return to 0° → advance to next run
- The feature is fully backward compatible (default is 1 run)
- Settings are persisted in NVS (non-volatile storage)
