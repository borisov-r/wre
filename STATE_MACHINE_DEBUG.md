# Rotary Encoder State Machine Debugging Guide

## Problem Report
Debug output shows pins changing but state machine not updating:
```
I (75147) wre: 🔍 DEBUG (Live): CLK=0 DT=1 State=0x00 Value=0 Angle=0.0°
I (75348) wre: 🔍 DEBUG (Live): CLK=1 DT=0 State=0x00 Value=0 Angle=0.0°
```

**Symptoms:**
- ✅ CLK pin changing (0 → 1)
- ✅ DT pin changing (1 → 0)
- ❌ State stuck at 0x00 (R_START)
- ❌ Value stuck at 0
- ❌ Angle stuck at 0.0°

## Diagnostic Tools Added

### New Debug Output Format
```
🔍 DEBUG: Live[CLK=X DT=Y] ISR[CLK=A DT=B Pins=0bXY] State=0xZZ Value=N Angle=A.A° ISR_Calls=NNN
```

**Fields Explained:**
- **Live[CLK=X DT=Y]**: Pin values read directly in main loop (current moment)
- **ISR[CLK=A DT=B]**: Pin values captured in last ISR call
- **Pins=0bXY**: 2-bit pin combination used for state machine lookup
  - 0b00: CLK=0, DT=0
  - 0b01: CLK=0, DT=1
  - 0b10: CLK=1, DT=0
  - 0b11: CLK=1, DT=1
- **State=0xZZ**: Current state machine state
- **Value=N**: Raw encoder value (half-steps)
- **Angle=A.A°**: Calculated angle
- **ISR_Calls=NNN**: Total number of ISR calls (interrupt counter)

## Diagnostic Scenarios

### Scenario 1: ISR Not Firing (ISR_Calls=0)
```
🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=0 Pins=0b00] State=0x00 Value=0 Angle=0.0° ISR_Calls=0
🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=0 DT=0 Pins=0b00] State=0x00 Value=0 Angle=0.0° ISR_Calls=0
```

**Diagnosis:** Interrupts not firing
**Indicators:**
- ISR_Calls stays at 0
- ISR pin values never update
- Pins value stuck at 0b00

**Possible Causes:**
1. Interrupt subscription failed
2. GPIO interrupt not enabled
3. Pin configuration issue
4. Hardware connection problem

**Fix Strategy:**
- Check clk.subscribe() and dt.subscribe() return values
- Verify interrupt configuration
- Test with simple ISR that just increments counter

### Scenario 2: ISR Firing But State Not Changing (ISR_Calls>0, State=0x00)
```
🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=1 Pins=0b01] State=0x00 Value=0 Angle=0.0° ISR_Calls=15
🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=1 DT=0 Pins=0b10] State=0x00 Value=0 Angle=0.0° ISR_Calls=23
```

**Diagnosis:** ISR running but state machine not transitioning
**Indicators:**
- ISR_Calls increasing
- ISR pins updating correctly
- Pins showing valid combinations (0b01, 0b10)
- State stuck at 0x00

**Possible Causes:**
1. State machine transition table incorrect
2. State masking issue
3. Mutex deadlock preventing state update
4. State not being stored to last_state

**Fix Strategy:**
- Review TRANSITION_TABLE_HALF_STEP
- Add logging inside process_pins
- Check state mask (STATE_MASK = 0x07)

### Scenario 3: ISR Firing, State Changing, But Value Not Changing
```
🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=1 Pins=0b01] State=0x02 Value=0 Angle=0.0° ISR_Calls=15
🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=1 DT=0 Pins=0b10] State=0x01 Value=0 Angle=0.0° ISR_Calls=23
```

**Diagnosis:** State machine working but not generating increments
**Indicators:**
- ISR_Calls increasing
- State changing (0x00 → 0x02 → 0x01)
- Value stuck at 0

**Possible Causes:**
1. Direction flags not set in transition table
2. Direction masking issue (DIR_MASK = 0x30)
3. Increment calculation wrong
4. Reverse flag canceling increments

**Fix Strategy:**
- Check DIR_CW (0x10) and DIR_CCW (0x20) in transition table
- Verify direction = state & DIR_MASK
- Check reverse flag (currently true)
- Log incr value before applying

### Scenario 4: Live and ISR Pins Differ (Timing Issue)
```
🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=0 DT=1 Pins=0b01] State=0x02 Value=5 Angle=2.5° ISR_Calls=87
```

**Diagnosis:** Normal behavior - different sampling times
**Indicators:**
- Live pins different from ISR pins
- This is expected and OK
- ISR captures at interrupt time
- Live reads at polling time (50-200ms later)

**Fix Strategy:**
- No fix needed - this is normal
- ISR pins are what matters for state machine

## State Machine Reference

### States
```
R_START  = 0x00  // Starting/idle state
R_CW_1   = 0x01  // Clockwise step 1
R_CW_2   = 0x02  // Clockwise step 2
R_CW_3   = 0x03  // Clockwise step 3
R_CCW_1  = 0x04  // Counter-clockwise step 1
R_CCW_2  = 0x05  // Counter-clockwise step 2
```

### Direction Flags
```
DIR_CW   = 0x10  // Clockwise direction detected
DIR_CCW  = 0x20  // Counter-clockwise direction detected
```

### Expected State Transitions (Clockwise)
```
R_START (0x00) --[0b11]--> R_START (0x00)
R_START (0x00) --[0b10]--> R_CW_1  (0x01)
R_CW_1  (0x01) --[0b00]--> R_CW_2  (0x02)
R_CW_2  (0x02) --[0b01]--> R_CW_3 | DIR_CW (0x13)
R_CW_3  (0x13) --[0b11]--> R_START | DIR_CW (0x10) → Value += 1
```

### Expected State Transitions (Counter-clockwise)
```
R_START (0x00) --[0b11]--> R_START (0x00)
R_START (0x00) --[0b01]--> R_CCW_1 (0x04)
R_CCW_1 (0x04) --[0b00]--> R_CCW_2 (0x05)
R_CCW_2 (0x05) --[0b10]--> R_CW_3 | DIR_CCW (0x23)
R_CW_3  (0x23) --[0b11]--> R_START | DIR_CCW (0x20) → Value -= 1
```

## Transition Table Reference

```rust
TRANSITION_TABLE_HALF_STEP: [[u8; 4]; 8]
// [state_index][pin_combination]
// pin_combination = (CLK << 1) | DT
//                    00     01     10     11
/* 0: R_START  */ [R_CW_3, R_CW_2, R_CW_1, R_START],
/* 1: R_CW_1   */ [R_CW_3|DIR_CCW, R_START, R_CW_1, R_START],
/* 2: R_CW_2   */ [R_CW_3|DIR_CW, R_CW_2, R_START, R_START],
/* 3: R_CW_3   */ [R_CW_3, R_CCW_2, R_CCW_1, R_START],
/* 4: R_CCW_1  */ [R_CW_3, R_CW_2, R_CCW_1, R_START|DIR_CW],
/* 5: R_CCW_2  */ [R_CW_3, R_CCW_2, R_CW_3, R_START|DIR_CCW],
/* 6: unused   */ [R_START, R_START, R_START, R_START],
/* 7: unused   */ [R_START, R_START, R_START, R_START],
```

## Testing Procedure

### Step 1: Check ISR Firing
1. Enable debug mode
2. Slowly rotate encoder
3. Watch ISR_Calls counter
4. **Expected**: Counter should increment
5. **If not**: ISR not firing - check interrupt configuration

### Step 2: Check Pin Combinations
1. Continue rotating slowly
2. Watch Pins value
3. **Expected**: Should see all combinations
   - 0b00 (CLK=0, DT=0)
   - 0b01 (CLK=0, DT=1)
   - 0b10 (CLK=1, DT=0)
   - 0b11 (CLK=1, DT=1)
4. **If not**: Hardware or reading issue

### Step 3: Check State Transitions
1. Continue rotating
2. Watch State value
3. **Expected**: Should transition through states
   - 0x00, 0x01, 0x02, 0x03, 0x04, 0x05
   - May have direction flags: 0x10, 0x20
4. **If stuck at 0x00**: State machine issue

### Step 4: Check Value Changes
1. Continue rotating
2. Watch Value
3. **Expected**: Should increment/decrement
4. **If stuck at 0**: Direction flag or increment issue

## Quick Reference

### Normal Operation Example
```
I (1000) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=1 DT=1 Pins=0b11] State=0x00 Value=0 Angle=0.0° ISR_Calls=1
[Rotate clockwise]
I (1200) wre: 🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=1 DT=0 Pins=0b10] State=0x01 Value=0 Angle=0.0° ISR_Calls=5
I (1400) wre: 🔍 DEBUG: Live[CLK=0 DT=0] ISR[CLK=0 DT=0 Pins=0b00] State=0x02 Value=0 Angle=0.0° ISR_Calls=9
I (1600) wre: 🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=1 Pins=0b01] State=0x13 Value=0 Angle=0.0° ISR_Calls=13
I (1800) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=1 DT=1 Pins=0b11] State=0x10 Value=1 Angle=0.5° ISR_Calls=17
```

**Analysis:**
- ✅ ISR_Calls incrementing (5, 9, 13, 17)
- ✅ Pins cycling through all combinations
- ✅ State transitioning (0x00→0x01→0x02→0x13→0x10)
- ✅ Value incrementing (0→1)
- ✅ Angle calculating (0.0°→0.5°)
- ✅ System working correctly!

### Broken ISR Example
```
I (1000) wre: 🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=0 Pins=0b00] State=0x00 Value=0 Angle=0.0° ISR_Calls=0
I (1200) wre: 🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=0 DT=0 Pins=0b00] State=0x00 Value=0 Angle=0.0° ISR_Calls=0
I (1400) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=0 DT=0 Pins=0b00] State=0x00 Value=0 Angle=0.0° ISR_Calls=0
```

**Analysis:**
- ❌ ISR_Calls stuck at 0
- ❌ ISR pins not updating
- ❌ Pins stuck at 0b00
- ❌ State stuck at 0x00
- ❌ **ISR NOT FIRING!**

## Next Steps

1. **Upload new firmware with diagnostics**
2. **Enable debug mode**
3. **Rotate encoder slowly**
4. **Check ISR_Calls counter**
   - If 0: ISR not firing → Check interrupt config
   - If >0: ISR working → Check state transitions
5. **Report findings** with debug output

This comprehensive diagnostic will definitively identify whether the issue is interrupt configuration or state machine logic.
