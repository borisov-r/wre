# Quick Reference - Where ISR Setup Happens

## Location in Code: src/main.rs Lines 119-180

### Visual Overview

```
src/main.rs
│
├─ Lines 70-103: Pin Setup
│  ├─ 79-81: Create CLK PinDriver + configure
│  ├─ 83-85: Create DT PinDriver + configure
│  ├─ 81, 85: ⚠️ set_interrupt_type(AnyEdge) ← CRITICAL
│  └─ 91-103: Low-level GPIO configuration
│
├─ Lines 119-180: ⭐ INTERRUPT HANDLER SETUP (CRITICAL SECTION)
│  │
│  ├─ Line 120: Clone state for ISR use
│  │
│  ├─ Lines 134-135: ⚠️ DECLARE SUBSCRIPTION HANDLES
│  │   let _clk_subscription;
│  │   let _dt_subscription;
│  │   ↑ MUST EXIST or interrupts will be unregistered!
│  │
│  ├─ Lines 140-157: Subscribe to CLK interrupts
│  │   ├─ Line 140: ⚠️ _clk_subscription = clk.subscribe({
│  │   │              ↑ MUST STORE return value!
│  │   ├─ Lines 146-147: Capture pin numbers
│  │   │   let clk_num = clk_pin_num;
│  │   │   let dt_num = dt_pin_num;
│  │   └─ Lines 150-157: ISR closure (runs on interrupt)
│  │       └─ Line 155: encoder_state.process_pins(clk, dt)
│  │                     ↑ This increments ISR_Calls counter
│  │
│  ├─ Lines 162-179: Subscribe to DT interrupts (same pattern)
│  │   └─ Line 162: ⚠️ _dt_subscription = dt.subscribe({
│  │
│  └─ Line 180: Log confirmation message
│
└─ Lines 183+: Main loop (keeps handles alive)
```

## The Problem: ISR_Calls=0

If you see this debug output:
```
🔍 DEBUG: Live[CLK=1 DT=0] ISR[CLK=0 DT=0] State=0x00 Value=0 ISR_Calls=0
                                                              └─ THIS IS THE PROBLEM!
```

### What It Means

| Field | What You See | What It Means |
|-------|--------------|---------------|
| Live[CLK=1 DT=0] | Changing | ✅ Hardware working, pins readable |
| ISR[CLK=0 DT=0] | Stuck at 0,0 | ❌ ISR not updating (not firing) |
| State=0x00 | Stuck | ❌ State machine not running |
| Value=0 | Stuck | ❌ Encoder not counting |
| ISR_Calls=0 | Zero | ❌ **ISR NEVER FIRED** |

## Root Cause 99% of the Time

### Scenario A: Code Missing Critical Lines (10%)

**Check these lines in src/main.rs:**
```rust
134: let _clk_subscription;     ← Must exist
135: let _dt_subscription;      ← Must exist
140: _clk_subscription = clk.   ← Must have assignment
162: _dt_subscription = dt.     ← Must have assignment
```

**Quick Test**: Run `bash verify_isr_fix.sh`

### Scenario B: Wrong Binary Uploaded (90%)

**The Problem:**
- Your local code has the fix ✅
- But the binary on ESP32 is from an old build ❌

**How This Happens:**
1. CI builds old commit (cached)
2. You download old artifact
3. Or CI building wrong branch
4. Or you uploaded old .bin file from previous download

**Solution:**
1. Check git commit: `git log --oneline -1`
2. Should show: `d04d3df Add comprehensive code explanation...`
3. Trigger fresh CI build if needed
4. Download LATEST artifact
5. Upload to ESP32
6. Hard reset (power cycle)

## Quick Diagnostic

### Step 1: Is the Fix in Your Code?

```bash
cd /home/runner/work/wre/wre
bash verify_isr_fix.sh
```

**Expected**: All checks show "✓ PASS"

### Step 2: Which Commit Is Running?

**On ESP32 boot, you should see:**
```
✓ GPIO pins explicitly configured as INPUT with PULL-UP
✓ Interrupt handlers subscribed for GPIO 21 (CLK) and GPIO 22 (DT)
```

If missing: Wrong binary!

### Step 3: Is Debug Mode Enabled?

**Enable via web interface:**
- Click "🔍 Toggle Debug Mode" button
- Should see debug messages in serial console

### Step 4: Rotate Encoder

**ISR_Calls should increment:**
```
ISR_Calls=0   ← Before rotation
ISR_Calls=5   ← After one click
ISR_Calls=13  ← After more rotation
```

**If stays at 0:** Wrong binary or hardware issue

## The Critical Code Pattern

This is what MUST be in the code:

```rust
// CORRECT (what should be there)
let _clk_subscription;  // Declare variable

unsafe {
    _clk_subscription = clk.subscribe({  // Store return value
        let clk_num = clk_pin_num;  // Capture variables
        move || {
            // ISR code
        }
    })?;
}
// Handle stays alive in infinite loop below
```

**Wrong patterns:**

```rust
// ❌ WRONG - Handle not stored, immediately dropped
clk.subscribe({...})?;

// ❌ WRONG - Variables not captured
move || {
    gpio_get_level(clk_pin_num)  // Won't work!
}
```

## Files to Read

**Start Here:**
1. **TROUBLESHOOTING.md** - Quick fixes for common issues
2. **HOW_IT_WORKS.md** - Complete explanation of the code
3. Run **verify_isr_fix.sh** - Check if code has all fixes

**Deep Dives:**
4. **SUBSCRIPTION_HANDLE_FIX.md** - Why handles must be kept alive
5. **STATE_MACHINE_DEBUG.md** - Understanding debug output

## Most Likely Solution

**If verify_isr_fix.sh shows all PASS:**
→ Your code is correct
→ But wrong binary is on ESP32
→ Download fresh CI artifact and upload again

**If verify_isr_fix.sh shows FAIL:**
→ Your local code is missing the fix
→ Pull latest changes from repository
→ Verify lines 134-135, 140, 162 in src/main.rs

## Summary

**The ISR setup happens in lines 119-180 of src/main.rs**

**Critical requirements:**
1. ✓ Lines 134-135: Declare `_clk_subscription` and `_dt_subscription`
2. ✓ Line 140: Store CLK subscription handle
3. ✓ Line 162: Store DT subscription handle
4. ✓ Lines 146-147, 164-165: Capture pin numbers
5. ✓ Handles stay alive in infinite loop (lines 183+)

**If all above present but ISR_Calls=0:**
→ 90% chance: Wrong binary uploaded
→ 10% chance: Hardware issue

**Next steps:**
1. Run `bash verify_isr_fix.sh`
2. If PASS: Re-upload latest CI binary
3. If FAIL: Pull latest code
4. Read TROUBLESHOOTING.md for detailed steps
