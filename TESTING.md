# Testing Guide

## Testing Without Physical Hardware

You can test most of the system without a physical rotary encoder:

### 1. Web Interface Testing

```bash
# Build and flash
export WIFI_SSID="YourNetwork"
export WIFI_PASS="YourPassword"
cargo run --release
```

The web interface will work fully even without encoder hardware:
- ✅ WiFi connection
- ✅ Web server and REST API
- ✅ Status updates (will show angle = 0°)
- ✅ Set target angles command
- ✅ Stop command
- ❌ Encoder rotation (requires physical hardware)

### 2. Serial Monitor Verification

Watch the serial output for:
```
🔧 ESP32 Rotary Encoder Control - Rust Edition
Starting dual-core application...
Initializing WiFi...
WiFi connected! IP: 192.168.1.xxx
Web server started at http://192.168.1.xxx
Rotary encoder task running on Core 1
```

### 3. API Testing with curl

```bash
# Get status
curl http://192.168.1.xxx/api/status

# Expected output:
# {"active":false,"angle":0.0,"target_angles":[],"current_target_index":0,"output_on":false}

# Set angles
curl -X POST http://192.168.1.xxx/api/set \
  -H "Content-Type: application/json" \
  -d '{"angles":[45, 90, 135]}'

# Stop encoder
curl -X POST http://192.168.1.xxx/api/stop
```

## Testing With Physical Hardware

### Full System Test

1. **Hardware Setup**
   - Connect rotary encoder (CLK=21, DT=22)
   - Connect output device (LED/relay on GPIO 32)
   - Power up ESP32

2. **Basic Rotation Test**
   ```bash
   # Flash firmware
   cargo run --release
   
   # Monitor serial output
   # Rotate encoder slowly
   # Should see: "⚡ Target reached: X.X°"
   ```

3. **Web Interface Test**
   - Open browser to ESP32 IP
   - Set target angles: "45, 90, 135"
   - Click "Start"
   - Rotate encoder to each angle
   - Verify output toggles at each target
   - Verify web interface shows:
     - Current angle updates in real-time
     - Output indicator changes color
     - Progress counter advances

4. **Reset Test**
   - Rotate encoder past first target
   - Output should turn ON
   - Rotate back below 2°
   - Output should turn OFF
   - Should advance to next target
   - Serial monitor: "🔄 Encoder reset to 0°"

5. **Multiple Target Test**
   - Set: "30, 60, 90, 120, 150, 180"
   - Rotate through all angles
   - Each should trigger output
   - Each reset should advance

### Interrupt Performance Test

```bash
# Monitor interrupt frequency
# Add this code to rotary_task for testing:
static mut interrupt_count: u32 = 0;
// In ISR: interrupt_count += 1;
// In main loop: print every second
```

Expected performance:
- Idle: ~0 interrupts/sec
- Slow rotation: ~50-200 interrupts/sec
- Fast rotation: ~500-2000 interrupts/sec
- No missed interrupts under 5000/sec

### Output Timing Test

Use an oscilloscope or logic analyzer on GPIO 32:
1. Set single target: "90"
2. Rotate encoder to 90°
3. Verify output HIGH exactly at 90°
4. Rotate back to 0°
5. Verify output LOW below 2°

### Stress Test

1. **Rapid Rotation Test**
   - Rotate encoder very fast
   - Should not lose steps
   - Web interface should keep up

2. **WiFi Stability Test**
   - Keep encoder active
   - Continuously refresh browser
   - Should not affect encoder operation
   - No Core 1 delays from Core 0 activity

3. **Long Running Test**
   - Set many targets
   - Run for hours
   - Monitor for:
     - Memory leaks (none expected)
     - WiFi disconnections
     - Missed interrupts
     - Incorrect angle tracking

## Unit Testing (Future Work)

Currently no automated tests. Future additions could include:

### Encoder State Machine Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clockwise_rotation() {
        let state = RotaryEncoderState::new(0, 720, false);
        // Simulate pin transitions
        // Verify value increments
    }
    
    #[test]
    fn test_bounded_range() {
        let state = RotaryEncoderState::new(0, 720, false);
        state.set_value(720);
        state.process_pins(true, true); // Try to increment
        assert_eq!(state.get_value(), 720); // Should clamp
    }
}
```

### API Tests
```rust
#[cfg(test)]
mod api_tests {
    #[test]
    fn test_status_endpoint() {
        // Mock encoder state
        // Call status endpoint
        // Verify JSON response
    }
}
```

## Debugging Tips

### Enable Verbose Logging

In `src/main.rs`:
```rust
esp_idf_svc::log::EspLogger::initialize_default();
// Add more detailed logging
log::set_max_level(log::LevelFilter::Debug);
```

### Monitor Core Assignment

```rust
// In each task
info!("Running on core: {}", esp_idf_hal::cpu::core());
```

### Track Mutex Contention

```rust
// Add timing around mutex locks
let start = std::time::Instant::now();
let lock = self.target_angles.lock().unwrap();
let duration = start.elapsed();
if duration.as_millis() > 10 {
    warn!("Mutex held for {}ms", duration.as_millis());
}
```

### Memory Usage

```rust
// Add to main loop
info!("Free heap: {} bytes", esp_idf_sys::esp_get_free_heap_size());
```

## Common Issues and Solutions

### Issue: Encoder counts wrong direction
**Solution**: Set `reverse: true` in RotaryEncoderState::new()

### Issue: Output triggers at wrong angle
**Solution**: Check if using degrees vs half-steps (multiply by 2)

### Issue: Web interface not updating
**Solution**: Check browser console, verify polling is working

### Issue: Interrupts too frequent
**Solution**: Add hardware debouncing (0.1µF capacitors on CLK/DT)

### Issue: ESP32 crashes on fast rotation
**Solution**: Increase stack size for rotary_task

### Issue: Output not working
**Solution**: Check GPIO 32 pin, verify output.set_high() is called

## Performance Benchmarks

Expected performance on ESP32 @ 240MHz:

| Metric | Value |
|--------|-------|
| Interrupt latency | < 10µs |
| State machine processing | < 5µs |
| Web request latency | < 50ms |
| Status update rate | 5 Hz (200ms) |
| Max encoder speed | ~5000 steps/sec |
| WiFi throughput | 1-2 Mbps (sufficient) |
| Power consumption | ~150mA @ 3.3V |
| Flash usage | ~1.8MB |
| RAM usage | ~100KB |

## Next Steps

Once basic testing passes:
1. Add automated unit tests
2. Add integration tests with mock hardware
3. Add CI/CD pipeline
4. Performance profiling with benchmarks
5. Power consumption optimization
6. OTA (Over-The-Air) update support
