# System Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                            ESP32                                 │
│                                                                   │
│  ┌─────────────────────────┐   ┌──────────────────────────┐    │
│  │     Core 0 (Protocol)   │   │   Core 1 (Application)   │    │
│  │                         │   │                          │    │
│  │  ┌─────────────────┐   │   │  ┌──────────────────┐   │    │
│  │  │  WiFi Manager   │   │   │  │  GPIO Polling    │   │    │
│  │  └────────┬────────┘   │   │  │   (~1000Hz)      │   │    │
│  │           │            │   │  └────────┬─────────┘   │    │
│  │  ┌────────▼────────┐   │   │           │             │    │
│  │  │  HTTP Server    │   │   │  ┌────────▼─────────┐   │    │
│  │  │                 │   │   │  │  Rotary Encoder  │   │    │
│  │  │  REST API       │◄──┼───┼──│  Library         │   │    │
│  │  │  /api/status    │   │   │  └────────┬─────────┘   │    │
│  │  │  /api/set       │   │   │           │             │    │
│  │  │  /api/stop      │   │   │  ┌────────▼─────────┐   │    │
│  │  └─────────────────┘   │   │  │  Output Control  │   │    │
│  │                         │   │  └────────┬─────────┘   │    │
│  └─────────────────────────┘   └───────────┼──────────────┘    │
│                                             │                    │
└─────────────────────────────────────────────┼────────────────────┘
                                              │
                        ┌─────────────────────┴─────────────────┐
                        │                                         │
                   ┌────▼────┐                              ┌────▼────┐
                   │ GPIO 21 │                              │ GPIO 32 │
                   │  (CLK)  │                              │ (Output)│
                   └────┬────┘                              └────┬────┘
                        │                                        │
                   ┌────▼────┐                              ┌────▼────┐
                   │ GPIO 22 │                              │ LED/    │
                   │  (DT)   │                              │ Relay   │
                   └────┬────┘                              └─────────┘
                        │
                  ┌─────▼──────┐
                  │   Rotary   │
                  │  Encoder   │
                  └────────────┘
```

## Data Flow

### 1. Encoder Input Flow (Core 1)

```
Rotary Encoder Rotation
         │
         ▼
GPIO Polling Loop (~1000Hz)
         │
         ▼
Read CLK and DT pin states
         │
         ▼
rotary-encoder-embedded processes states
         │
         ▼
Returns Direction (CW/CCW/None)
         │
         ▼
Atomic update of encoder value
         │
         ▼
Check if target reached
         │
         ▼
Update output pin (GPIO 32)
```

### 2. Web Request Flow (Core 0)

```
Browser HTTP Request
         │
         ▼
WiFi → HTTP Server
         │
         ├─────GET /─────────────► Serve HTML
         │
         ├─────GET /api/status──► Read encoder state (atomic)
         │                        ├─ Current angle
         │                        ├─ Target angles
         │                        ├─ Active status
         │                        └─ Output ON/OFF
         │
         ├─────POST /api/set────► Update target angles (mutex)
         │                        └─ Activate encoder
         │
         └─────POST /api/stop───► Deactivate encoder (atomic)
                                  └─ Turn off output
```

### 3. Shared State Management

```
┌───────────────────────────────────────────────────┐
│           Encoder State (Shared)                  │
├───────────────────────────────────────────────────┤
│                                                   │
│  Arc<AtomicI32>          value                   │  ◄── Lock-free
│  Arc<AtomicBool>         encoder_active          │  ◄── Lock-free
│  Arc<AtomicBool>         output_on               │  ◄── Lock-free
│  Arc<AtomicBool>         triggered               │  ◄── Lock-free
│  Arc<AtomicBool>         reset_detected          │  ◄── Lock-free
│                                                   │
│  Arc<Mutex<Vec<i32>>>    target_angles           │  ◄── Mutex protected
│  Arc<Mutex<usize>>       current_target_index    │  ◄── Mutex protected
│                                                   │
└───────────────────────────────────────────────────┘
         ▲                              ▲
         │                              │
    Core 0 reads                   Core 1 writes
   (no contention)                (polling loop)
```

## Encoder Processing

The rotary encoder is processed using the [rotary-encoder-embedded](https://github.com/ost-ing/rotary-encoder-embedded) library:

```
Polling Loop (1ms interval)
         │
         ▼
Read CLK and DT pins
         │
         ▼
StandardMode.update(dt, clk)
         │
         ▼
Returns Direction
   ├── Clockwise → Increment value
   ├── Anticlockwise → Decrement value
   └── None → No change
```

## Timing Diagram

```
Time →

Encoder Input (CLK):  ──┐    ┌────┐    ┌────
                        └────┘    └────┘

Encoder Input (DT):   ────┐    ┌────┐    ┌──
                          └────┘    └────┘

Polling Events:       ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
                      (Every 1ms, ~1000Hz)

Value Updates:        0 → 0 → 1 → 1 → 2 → 2 → 3
                      (Half-steps incremented on valid transitions)

Output (GPIO 32):     ────────────┌────────────
                      (HIGH when value >= target)

Web Update (200ms):   ↓        ↓        ↓
                      (Polls status every 200ms)
```

## Memory Layout

```
Flash (ROM)                    RAM (DRAM)
┌──────────────────┐          ┌──────────────────┐
│                  │          │                  │
│  Application     │          │  Heap            │
│  Code            │          │  ┌────────────┐ │
│  ~500KB          │          │  │ WiFi Stack │ │
│                  │          │  │  ~50KB     │ │
│  HTML/Assets     │          │  └────────────┘ │
│  ~10KB           │          │  ┌────────────┐ │
│                  │          │  │ HTTP Stack │ │
│  ESP-IDF         │          │  │  ~30KB     │ │
│  Libraries       │          │  └────────────┘ │
│  ~1MB            │          │  ┌────────────┐ │
│                  │          │  │ App Data   │ │
│  WiFi Firmware   │          │  │  ~10KB     │ │
│  ~500KB          │          │  └────────────┘ │
│                  │          │                  │
└──────────────────┘          └──────────────────┘
                              
IRAM (Instruction RAM)         RTC Memory
┌──────────────────┐          ┌──────────────────┐
│  ISR Code        │          │  Deep Sleep Data │
│  Critical Paths  │          │  (unused)        │
│  ~20KB           │          └──────────────────┘
└──────────────────┘
```

## Concurrency Model

```
┌─────────────────────────────────────────────────┐
│              FreeRTOS Scheduler                 │
└─────────┬───────────────────────┬───────────────┘
          │                       │
    ┌─────▼──────┐          ┌─────▼──────┐
    │  Core 0    │          │  Core 1    │
    │  Priority  │          │  Priority  │
    │    5       │          │    5       │
    └─────┬──────┘          └─────┬──────┘
          │                       │
    ┌─────▼──────────────┐  ┌─────▼──────────────┐
    │ WiFi Task          │  │ Rotary Task        │
    │ HTTP Task          │  │ (Main Loop)        │
    │ TCP/IP Task        │  │                    │
    └────────────────────┘  └────────────────────┘
                                   │
                            ┌──────▼──────┐
                            │ ISR (Level 1)│
                            │ GPIO Handler │
                            └─────────────┘
```

## API Sequence Diagram

```
Browser          HTTP Server (Core 0)    Encoder State    Rotary Task (Core 1)
   │                    │                      │                  │
   │──GET /api/status──►│                      │                  │
   │                    │──read (atomic)──────►│                  │
   │                    │◄─────angle───────────│                  │
   │                    │◄─────status──────────│                  │
   │◄────JSON response──│                      │                  │
   │                    │                      │                  │
   │──POST /api/set────►│                      │                  │
   │  {angles:[45,90]} │                      │                  │
   │                    │──write (mutex)──────►│                  │
   │                    │                      │──notify─────────►│
   │◄────200 OK────────│                      │                  │
   │                    │                      │                  │
   │                    │                      │◄──update value───│
   │                    │                      │◄──set output─────│
   │                    │                      │                  │
```

This architecture ensures:
- **High Update Rate**: 1000Hz polling for responsive encoder tracking
- **Thread Safety**: Atomic operations and mutexes prevent race conditions
- **Responsiveness**: Web server doesn't block encoder processing
- **Reliability**: Each core can operate independently
- **Library-Based**: Uses well-tested rotary-encoder-embedded library
- **Scalability**: Easy to add more endpoints or encoder features
