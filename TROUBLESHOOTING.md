# Quick Troubleshooting Guide - ISR Not Firing

## Your Issue: CLK and DT Change, But ISR_Calls=0

You're seeing the Live pins change in debug output, but the ISR (Interrupt Service Routine) is not being called. This means the encoder hardware is working, but the software interrupt handler is not firing.

## Most Likely Causes

### 1. Wrong Binary Uploaded (90% of cases)

**Problem**: The binary you uploaded to ESP32 doesn't have the fix.

**How to Check**:
- Look at the git commit hash in the CI/Build logs
- Compare to the commit hash in your repository
- Run: `cd /home/runner/work/wre/wre && git log --oneline -1`
- Check if that commit is what you uploaded

**Solution**:
1. Download the **latest** artifact from CI/Build
2. Make sure you're downloading from the correct branch (copilot/add-debug-button-for-encoder)
3. Upload the correct .bin file to your ESP32
4. Reset the ESP32 after upload

### 2. Code Missing Critical Lines

**Problem**: The subscription handles aren't being stored, so interrupts are immediately unregistered.

**How to Check**:
Run the verification script:
```bash
cd /home/runner/work/wre/wre
bash verify_isr_fix.sh
```

**What to look for**:
- All checks should show "✓ PASS"
- If any show "✗ FAIL", that's your problem

**Critical lines that MUST exist** (in src/main.rs):
```
Line 134: let _clk_subscription;
Line 135: let _dt_subscription;
Line 140: _clk_subscription = clk.subscribe({
Line 162: _dt_subscription = dt.subscribe({
```

### 3. CI/Build Using Old Code

**Problem**: CI might be building from an old commit.

**How to Check**:
1. Go to GitHub Actions for your repository
2. Find the latest successful build
3. Check which commit it built
4. Verify that commit has the fixes

**Solution**:
- Trigger a new CI build from the latest commit
- Make sure the branch has all the latest changes
- Download the artifact from the NEW build

## Detailed Verification Steps

### Step 1: Verify Your Local Code

```bash
cd /home/runner/work/wre/wre

# Check commit
git log --oneline -1

# Should show something like:
# 58fa3ac Add comprehensive documentation for subscription handle lifetime fix

# Run verification
bash verify_isr_fix.sh

# All checks should PASS
```

### Step 2: Verify CI Build

1. Go to: https://github.com/borisov-r/wre/actions
2. Look for the latest workflow run
3. Click on it
4. Check "Build for ESP32" step
5. Verify it shows the correct commit hash
6. Download the artifact

### Step 3: Verify Binary Upload

1. Make sure you're using the right USB port
2. Use correct flash command (esptool or platform.io)
3. After upload, reset the ESP32
4. Connect serial monitor at 115200 baud
5. Look for startup messages

### Step 4: Check Serial Output

**What you SHOULD see on boot:**
```
I (1234) wre: ✓ GPIO pins explicitly configured as INPUT with PULL-UP
I (1235) wre: 📌 Pin configuration verified - CLK: HIGH (1), DT: HIGH (1)
I (1236) wre: ✓ Interrupt handlers subscribed for GPIO 21 (CLK) and GPIO 22 (DT)
```

**Then enable debug mode via web interface.**

**What you SHOULD see when encoder rotates:**
```
I (2000) wre: 🔍 DEBUG: Live[CLK=1 DT=1] ISR[...] ISR_Calls=1
I (2200) wre: 🔍 DEBUG: Live[CLK=1 DT=0] ISR[...] ISR_Calls=5
I (2400) wre: 🔍 DEBUG: Live[CLK=0 DT=0] ISR[...] ISR_Calls=9
```

**If ISR_Calls stays at 0**: Wrong binary uploaded or code doesn't have fix

## Common Upload Issues

### Issue A: Using Old Binary

**Symptom**: You think you uploaded latest but ISR_Calls=0

**Fix**:
1. Delete all downloaded .bin files
2. Go to GitHub → Actions → Latest successful run
3. Download artifact again
4. Verify file timestamp is recent
5. Upload to ESP32
6. Hard reset (unplug power)

### Issue B: Cache Problem

**Symptom**: CI shows "Using cached build"

**Fix**:
1. Make a small change (add comment)
2. Commit and push
3. Wait for new CI build
4. Download NEW artifact

### Issue C: Wrong Flash Address

**Symptom**: Binary uploads but doesn't run correctly

**Fix**:
Use correct esptool command:
```bash
esptool.py --chip esp32 --port /dev/ttyUSB0 write_flash 0x10000 firmware.bin
```

## How to Know If It's Working

### 1. Boot Messages

You MUST see:
```
✓ GPIO pins explicitly configured as INPUT with PULL-UP
✓ Interrupt handlers subscribed for GPIO 21 (CLK) and GPIO 22 (DT)
```

If missing: Wrong binary or boot failed

### 2. Debug Mode

Enable via web interface button "🔍 Toggle Debug Mode"

### 3. ISR_Calls Counter

Watch serial output. When you rotate encoder:
- ISR_Calls should increment
- Should increase by several for each click
- Usually jumps by 4-8 per detent

### 4. Value/Angle Changes

After ISR_Calls increments:
- Value should change (0, 1, 2, ...)
- Angle should change (0.0°, 0.5°, 1.0°, ...)

## If Still Not Working

### Last Resort Checklist

1. ✓ Verified code has all critical lines (run verify_isr_fix.sh)
2. ✓ Downloaded latest artifact from CI
3. ✓ Uploaded correct .bin file
4. ✓ Hard reset ESP32 after upload
5. ✓ Serial monitor connected at 115200 baud
6. ✓ See boot messages with "✓ Interrupt handlers subscribed"
7. ✓ Enabled debug mode via web interface
8. ✓ Rotated encoder while watching serial output

If ALL of above done and ISR_Calls still 0:

### Possible Hardware Issues

- Check encoder connections (should have 5 wires: GND, +, CLK, DT, SW)
- Verify CLK → GPIO21, DT → GPIO22
- Try different encoder (may be faulty)
- Check if encoder has pull-up resistors (some need external)

### Possible Software Issues

- ESP-IDF version mismatch
- Different board variant (ESP32 vs ESP32-S3 etc)
- GPIO conflict with other peripherals

## Need More Help?

1. Run verification script and paste output
2. Show the serial console output from boot
3. Show the git commit hash you're using
4. Show the CI build logs
5. Confirm binary file size (should be several hundred KB)

## Files to Reference

- **HOW_IT_WORKS.md**: Complete explanation of the code
- **SUBSCRIPTION_HANDLE_FIX.md**: Details about the RAII pattern
- **STATE_MACHINE_DEBUG.md**: Debug output interpretation
- **verify_isr_fix.sh**: Quick code verification script
