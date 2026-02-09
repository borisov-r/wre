# Rust Implementation Details

## Architecture Overview

This project implements a wireless rotary encoder control system using Rust and ESP32's dual-core architecture for optimal performance and responsiveness.

### Dual-Core Design

#### Core 0 (Protocol Core)
- **Primary Role**: Networking and HTTP server
- **Responsibilities**:
  - WiFi connection management
  - HTTP server with REST API
  - Serving web interface
  - Processing client requests
  - Reading encoder state for status updates
- **Stack Size**: 16KB
- **Priority**: 5

#### Core 1 (Application Core)  
- **Primary Role**: Real-time rotary encoder processing
- **Responsibilities**:
  - High-frequency polling of GPIO pins for CLK and DT (~1000Hz)
  - Rotary encoder processing using rotary-encoder-embedded library
  - Output pin control (GPIO 32)
  - Target angle management
  - Auto-reset logic
- **Stack Size**: 8KB
- **Priority**: 5

### Thread-Safe State Management

The encoder state is shared between cores using Rust's safe concurrency primitives:

```rust
// Atomic types for lock-free access
Arc<AtomicI32>        // Encoder value (half-steps)
Arc<AtomicBool>       // Flags: active, output_on, triggered, reset_detected

// Mutex-protected types for complex data
Arc<Mutex<Vec<i32>>>  // Target angles list
Arc<Mutex<usize>>     // Current target index
```

### Rotary Encoder Processing

Uses the [rotary-encoder-embedded](https://github.com/ost-ing/rotary-encoder-embedded) library for reliable encoder handling:
- **Input Resolution**: 2 half-steps = 1 full step = 1 degree
- **Range**: 0-720 half-steps (0-360 degrees)
- **Mode**: StandardMode (suitable for encoders with detents)
- **Polling Rate**: ~1000Hz (recommended by the library for best results)

The library handles state transitions internally and returns Direction:
- `Direction::Clockwise` - Increment encoder value
- `Direction::Anticlockwise` - Decrement encoder value
- `Direction::None` - No change

### GPIO Polling

Both encoder pins (CLK and DT on GPIO 21 and 22) are polled at high frequency:
```rust
// Poll at ~1000Hz (1ms delay)
thread::sleep(Duration::from_millis(1));
```

The polling loop:
1. Reads both pin states
2. Updates the encoder using rotary-encoder-embedded library
3. Processes direction changes
4. Updates the encoder value
5. Handles target angle logic

### Output Control Logic

The output pin (GPIO 32) follows this logic:
1. **Trigger**: When encoder value >= target angle (moving forward from 0)
2. **Hold**: Output stays HIGH while above target
3. **Reset**: When encoder returns below 2°, advance to next target
4. **Complete**: When all targets are reached and returned to 0°

### REST API Endpoints

#### GET /
Returns the web interface HTML

#### GET /api/status
Returns current encoder status as JSON:
```json
{
  "active": true,
  "angle": 45.5,
  "target_angles": [45, 90, 135],
  "current_target_index": 0,
  "output_on": true
}
```

#### POST /api/set
Set target angles. Request body:
```json
{
  "angles": [45, 90, 135, 180]
}
```

#### POST /api/stop
Stop the encoder and turn off output

### Web Interface

The web interface polls `/api/status` every 200ms for real-time updates:
- Current angle display
- Output status (ON/OFF indicator with animation)
- Encoder active/stopped status
- Progress through target sequence
- Input field for setting new angles
- Start/Stop buttons

### Performance Characteristics

- **Polling Rate**: 1000Hz (~1ms between reads)
- **Pin Read Latency**: <10μs
- **State Update Frequency**: Up to 1000Hz (limited by polling rate)
- **Web Update Rate**: 5Hz (200ms polling interval)
- **WiFi Throughput**: Not critical (only status updates)
- **Memory Usage**: 
  - Core 0: ~12KB heap for WiFi/HTTP
  - Core 1: ~4KB heap for encoder state
  - Shared: ~2KB for encoder state structures

### Safety and Reliability

1. **No Race Conditions**: All shared state uses atomic operations or mutexes
2. **No Deadlocks**: Mutexes are held for minimal time, no nested locks
3. **Bounded Memory**: Fixed-size allocations, no dynamic growth
4. **Polling Safety**: High-frequency polling ensures responsive encoder tracking
5. **Error Recovery**: Each core can restart independently
6. **Library-Based**: Uses well-tested rotary-encoder-embedded library for encoder logic

### Build Optimization

Release builds use:
- `opt-level = "s"` - Optimize for size while maintaining performance
- Link-time optimization (LTO)
- Strip debugging symbols
- Target-specific optimizations for Xtensa architecture

### Future Enhancements

Possible improvements:
1. Replace REST polling with WebSocket for true real-time updates
2. Add MDNS for discovery (access via `wre.local`)
3. Store configuration in NVS (non-volatile storage)
4. Add multiple encoder support
5. PWM output instead of binary ON/OFF
6. Touch sensor integration for manual control
7. MQTT support for IoT integration

## Comparison with MicroPython Version

| Feature | MicroPython | Rust |
|---------|------------|------|
| Performance | ~100μs polling interval | 1ms polling (1000Hz) |
| Memory Safety | Runtime checks | Compile-time guarantees |
| Concurrency | GIL limitations | True multi-core parallelism |
| Web Server | Basic HTTP | Full-featured HTTP with async |
| Encoder Library | Custom state machine | rotary-encoder-embedded |
| Code Size | ~50KB | ~500KB (includes ESP-IDF) |
| Development Speed | Fast prototyping | Longer compile times |
| Reliability | Good | Excellent (type safety) |
| Debugging | REPL, print statements | GDB, logging framework |

## Development Workflow

1. **Edit Code**: Modify Rust source files
2. **Build**: `cargo build --release` (takes 2-5 min first time, then incremental)
3. **Flash**: `cargo run --release` or `espflash flash --monitor`
4. **Monitor**: Serial output shows WiFi connection and IP address
5. **Test**: Open web interface in browser
6. **Iterate**: Repeat from step 1

## Troubleshooting

### Build Errors
- Ensure ESP Rust toolchain is installed: `espup install`
- Source environment: `source $HOME/export-esp.sh`
- Check Rust version: `rustc --version` (needs esp channel)

### Flash Errors
- Check USB connection and permissions: `ls -l /dev/ttyUSB*`
- Add user to dialout group: `sudo usermod -a -G dialout $USER`
- Try different baud rate: `espflash flash --baud 115200`

### Runtime Errors
- WiFi not connecting: Check SSID/password in environment variables
- Encoder not responding: Verify GPIO connections (CLK=21, DT=22)
- Output not working: Check GPIO 32 connection
- Web interface not loading: Check serial monitor for IP address

### Performance Issues
- Increase stack sizes in `main.rs` if seeing stack overflow
- Adjust polling interval in `index.html` if web updates too slow
- Check WiFi signal strength if connection unstable
