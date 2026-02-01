# How the Rotary Encoder Code Works - Complete Explanation

## Problem You're Experiencing

You're seeing CLK and DT pins changing in debug output, but ISR_Calls stays at 0 and nothing else updates. This means the **interrupt handler (ISR) is not firing**.

## Where This Happens in the Code

### Critical Section: Lines 122-155 in src/main.rs

This is where interrupt handlers are set up. Let me explain each part:

```rust
// Line 119-120: Create shared state that ISR can access
let encoder_state_isr = encoder_state.clone();

// Lines 122-126: Set up interrupt handlers
// CRITICAL: These variables hold subscription handles
// If they're dropped, interrupts stop working!
let _clk_subscription;
let _dt_subscription;

unsafe {
    // Line 128-139: Subscribe to CLK pin interrupts
    _clk_subscription = clk.subscribe({
        // Clone state for this closure
        let encoder_state = encoder_state_isr.clone();
        
        // Capture pin numbers (MUST be explicitly captured!)
        let clk_num = clk_pin_num;
        let dt_num = dt_pin_num;
        
        // This closure runs when CLK pin changes
        move || {
            // Read BOTH pins (needed for state machine)
            let clk_val = esp_idf_sys::gpio_get_level(clk_num) != 0;
            let dt_val = esp_idf_sys::gpio_get_level(dt_num) != 0;
            
            // Update state machine (THIS INCREMENTS ISR_Calls)
            encoder_state.process_pins(clk_val, dt_val);
        }
    })?;

    // Line 141-152: Subscribe to DT pin interrupts (same pattern)
    _dt_subscription = dt.subscribe({...})?;
}

// Line 155: Confirmation message
info!("✓ Interrupt handlers subscribed...");
```

## Why ISR Might Not Fire

### Issue 1: Subscription Handles Dropped (MOST COMMON)

**Symptom**: ISR_Calls=0, pins change but no response

**Problem**: The subscription handles MUST be kept alive. If you see this pattern in your code:

```rust
// ❌ WRONG - Handle immediately dropped!
clk.subscribe({...})?;

// ✅ CORRECT - Handle kept alive
let _clk_subscription = clk.subscribe({...})?;
```

**Location**: Lines 124-125 and 128, 141
**Fix**: Ensure lines 124-125 exist and assignments on lines 128, 141 store the return value

### Issue 2: Variables Not Captured in Closure

**Symptom**: Compile error or ISR doesn't access correct pins

**Problem**: Pin numbers must be explicitly captured:

```rust
// ❌ WRONG - clk_pin_num not captured
move || {
    let val = esp_idf_sys::gpio_get_level(clk_pin_num) != 0;
}

// ✅ CORRECT - Explicitly captured
let clk_num = clk_pin_num;
move || {
    let val = esp_idf_sys::gpio_get_level(clk_num) != 0;
}
```

**Location**: Lines 130-131, 143-144
**Fix**: These lines MUST exist before the `move ||` closure

### Issue 3: Interrupt Type Not Set

**Symptom**: ISR_Calls=0, no compiler errors

**Problem**: Pins must be configured for interrupts:

```rust
// Lines 81, 85 in src/main.rs
clk.set_interrupt_type(InterruptType::AnyEdge)?;
dt.set_interrupt_type(InterruptType::AnyEdge)?;
```

**Location**: Lines 81, 85
**Fix**: These MUST be called before subscribe()

## Complete Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│ 1. SETUP PHASE (Lines 79-103)                          │
├─────────────────────────────────────────────────────────┤
│ • Create PinDriver for CLK (GPIO21)                     │
│ • Create PinDriver for DT (GPIO22)                      │
│ • Set pull-up resistors                                 │
│ • Set interrupt type to AnyEdge ← CRITICAL!            │
│ • Configure GPIO at low level                           │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ 2. INTERRUPT SUBSCRIPTION (Lines 122-153)              │
├─────────────────────────────────────────────────────────┤
│ • Clone encoder_state for ISR use                       │
│ • Declare subscription handle variables ← CRITICAL!     │
│ • Subscribe to CLK interrupts:                          │
│   - Capture pin numbers explicitly                      │
│   - Store handle in _clk_subscription                   │
│ • Subscribe to DT interrupts:                           │
│   - Capture pin numbers explicitly                      │
│   - Store handle in _dt_subscription                    │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ 3. MAIN LOOP (Lines 158+)                              │
├─────────────────────────────────────────────────────────┤
│ • Subscription handles stay alive (infinite loop)        │
│ • Main loop polls for debug info                        │
│ • ISR runs independently when pins change               │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ 4. WHEN PIN CHANGES (Hardware Event)                   │
├─────────────────────────────────────────────────────────┤
│ Hardware detects CLK or DT pin change                   │
│         ↓                                                │
│ ESP-IDF calls registered ISR                            │
│         ↓                                                │
│ Our closure executes:                                   │
│ • Increments ISR_Calls counter                          │
│ • Reads both CLK and DT pin states                      │
│ • Calls encoder_state.process_pins(clk, dt)            │
│         ↓                                                │
│ State machine processes in src/rotary.rs                │
│         ↓                                                │
│ Value/Angle updated                                     │
└─────────────────────────────────────────────────────────┘
```

## Step-by-Step Verification

### Step 1: Check Subscription Handles

**Look at lines 124-125:**
```rust
let _clk_subscription;
let _dt_subscription;
```

✅ These lines MUST exist
❌ If missing, add them before the unsafe block

### Step 2: Check Handle Storage

**Look at lines 128 and 141:**
```rust
_clk_subscription = clk.subscribe({...})?;
_dt_subscription = dt.subscribe({...})?;
```

✅ Return value MUST be assigned to variable
❌ If it just says `clk.subscribe({...})?;` without assignment, that's the bug!

### Step 3: Check Variable Capture

**Look at lines 130-131 and 143-144:**
```rust
let clk_num = clk_pin_num;
let dt_num = dt_pin_num;
```

✅ These MUST be inside the closure scope but before `move ||`
❌ If missing, closure won't have access to pin numbers

### Step 4: Check Interrupt Type

**Look at lines 81 and 85:**
```rust
clk.set_interrupt_type(InterruptType::AnyEdge)?;
dt.set_interrupt_type(InterruptType::AnyEdge)?;
```

✅ MUST be called
❌ Without this, hardware won't trigger interrupts

## Common Mistakes and How to Fix

### Mistake 1: Not Storing Subscription Handle

**What you might see in code:**
```rust
unsafe {
    clk.subscribe({...})?;  // ← No assignment!
}
```

**How to fix:**
```rust
let _clk_subscription;
unsafe {
    _clk_subscription = clk.subscribe({...})?;  // ← Store it!
}
```

### Mistake 2: Variables Declared in Wrong Scope

**What you might see in code:**
```rust
let clk_num = 21;  // Outside closure
clk.subscribe({
    move || {
        gpio_get_level(clk_num)  // Won't work!
    }
})
```

**How to fix:**
```rust
clk.subscribe({
    let clk_num = clk_pin_num;  // Inside closure scope
    move || {
        gpio_get_level(clk_num)  // Works!
    }
})
```

### Mistake 3: Using ? Without Checking Error

**Problem**: If subscribe() returns an error, the `?` operator exits the function early, but you never see the error.

**How to check:**
```rust
// Add logging before and after
info!("Subscribing to CLK...");
_clk_subscription = clk.subscribe({...})?;
info!("✓ CLK subscribed successfully");
```

If you don't see both messages, subscribe failed!

## What Should Happen When Working

### 1. Boot Sequence
```
I (1234) wre: ✓ GPIO pins explicitly configured as INPUT with PULL-UP
I (1235) wre: 📌 Pin configuration verified - CLK: HIGH (1), DT: HIGH (1)
I (1236) wre: ✓ Interrupt handlers subscribed for GPIO 21 (CLK) and GPIO 22 (DT)
```

### 2. Debug Mode Active
```
I (2000) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=1 DT=1 Pins=0b11] State=0x00 Value=0 ISR_Calls=1
```
Note: ISR_Calls should be > 0 immediately (ISR fires on boot)

### 3. Rotating Encoder
```
I (2200) wre: 🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=1 DT=0 Pins=0b10] State=0x01 ISR_Calls=5
I (2400) wre: 🔍 DEBUG: Live[CLK=0 DT=0] ISR[CLK=0 DT=0 Pins=0b00] State=0x02 ISR_Calls=9
I (2600) wre: 🔍 DEBUG: Live[CLK=0 DT=1] ISR[CLK=0 DT=1 Pins=0b01] State=0x13 ISR_Calls=13
I (2800) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[CLK=1 DT=1 Pins=0b11] State=0x10 Value=1 Angle=0.5° ISR_Calls=17
```

## Debugging Checklist

When ISR_Calls=0 and pins are changing:

- [ ] Check that lines 124-125 exist (subscription handle variables declared)
- [ ] Check that line 128 has `_clk_subscription =` (not just `clk.subscribe`)
- [ ] Check that line 141 has `_dt_subscription =` (not just `dt.subscribe`)
- [ ] Check that lines 130-131 exist (variable capture in CLK closure)
- [ ] Check that lines 143-144 exist (variable capture in DT closure)
- [ ] Check that line 81 exists (CLK interrupt type set)
- [ ] Check that line 85 exists (DT interrupt type set)
- [ ] Check serial output for "✓ Interrupt handlers subscribed" message
- [ ] Check if subscription might be failing (add error logging)

## The Code Path When ISR Fires

```
1. Hardware detects pin change (e.g., CLK goes LOW→HIGH)
   ↓
2. ESP32 interrupt controller triggers
   ↓
3. ESP-IDF calls registered handler (our closure at line 133)
   ↓
4. Line 135: Read CLK pin state
   ↓
5. Line 136: Read DT pin state
   ↓
6. Line 137: Call encoder_state.process_pins(clk_val, dt_val)
   ↓
7. In src/rotary.rs, line 166: Increment ISR_Calls counter
   ↓
8. Lines 174-177: Calculate pin combination (CLK<<1 | DT)
   ↓
9. Lines 179-206: State machine logic
   ↓
10. Line 203: Update encoder value
   ↓
11. Main loop sees updated ISR_Calls, value, angle
```

## Why You Might Not See This Working

### Scenario A: Code Not Built Correctly

**Check**: What commit is actually running on your device?
**Solution**: Look at CI/Build logs, verify the correct commit was built

### Scenario B: Wrong Binary Uploaded

**Check**: Did you upload the Release or Debug binary?
**Solution**: Verify you uploaded the right .bin file to the ESP32

### Scenario C: Code Reverted by Accident

**Check**: View src/main.rs lines 124-125, 128, 141
**Solution**: If these don't match the documented code, the fix isn't present

### Scenario D: ESP-IDF Version Issue

**Check**: CI might be using different ESP-IDF version
**Solution**: Verify ESP-IDF version matches what code expects

## Quick Fix Verification Script

Run this to check if code has the fix:

```bash
# Check for subscription handle variables (should find 2 lines)
grep -n "let _.*_subscription;" src/main.rs

# Check for handle assignment (should find 2 lines with =)
grep -n "_.*_subscription = .*subscribe" src/main.rs

# Check for variable capture (should find 4 lines)
grep -n "let clk_num = clk_pin_num" src/main.rs
grep -n "let dt_num = dt_pin_num" src/main.rs
```

**Expected output:**
```
124:    let _clk_subscription;
125:    let _dt_subscription;
128:        _clk_subscription = clk.subscribe({
141:        _dt_subscription = dt.subscribe({
130:            let clk_num = clk_pin_num;
131:            let dt_num = dt_pin_num;
143:            let clk_num = clk_pin_num;
144:            let dt_num = dt_pin_num;
```

If you don't see these lines, **the fix is not in your code**!

## Summary

The ISR not firing is almost always due to one of these:

1. **Subscription handles not stored** (lines 124-125, 128, 141)
2. **Variables not explicitly captured** (lines 130-131, 143-144)
3. **Interrupt type not set** (lines 81, 85)
4. **Wrong binary uploaded** (not using latest build)

The most common issue is #1 - the subscription handles must be stored in variables that live for the entire function. Without this, the interrupts are unregistered immediately after subscription.
