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
│  │  │  WiFi Manager   │   │   │  │  GPIO Interrupt  │   │    │
│  │  └────────┬────────┘   │   │  │     Handler      │   │    │
│  │           │            │   │  └────────┬─────────┘   │    │
│  │  ┌────────▼────────┐   │   │           │             │    │
│  │  │  HTTP Server    │   │   │  ┌────────▼─────────┐   │    │
│  │  │                 │   │   │  │  Rotary Encoder  │   │    │
│  │  │  REST API       │◄──┼───┼──│  State Machine   │   │    │
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
GPIO Interrupt (CLK/DT edges)
         │
         ▼
Interrupt Handler reads pin states
         │
         ▼
State Machine processes transition
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
│  Arc<Mutex<u8>>          state                   │  ◄── Mutex protected
│                                                   │
└───────────────────────────────────────────────────┘
         ▲                              ▲
         │                              │
    Core 0 reads                   Core 1 writes
   (no contention)                (in ISR context)
```

## State Machine Diagram

```
Rotary Encoder Half-Step State Machine

Initial State: R_START (0x0)

  CLK  DT    Next State       Action
  ─────────────────────────────────────
   0   0  →  R_CW_3           -
   0   1  →  R_CW_2           -
   1   0  →  R_CW_1           -
   1   1  →  R_START          -

From R_CW_1:
   0   0  →  R_CW_2           -
   0   1  →  R_START          -
   1   0  →  R_CW_1           -
   1   1  →  R_START          -

From R_CW_2:
   0   0  →  R_CW_2           -
   0   1  →  R_CW_3           -
   1   0  →  R_CW_1           -
   1   1  →  R_START          -

From R_CW_3 (Clockwise Complete):
   0   0  →  R_CW_2           -
   0   1  →  R_CW_3           -
   1   0  →  R_START          -
   1   1  →  R_START          Increment (+1 half-step)

(Similar transitions for CCW direction)
```

## Timing Diagram

```
Time →

Encoder Input (CLK):  ──┐    ┌────┐    ┌────
                        └────┘    └────┘

Encoder Input (DT):   ────┐    ┌────┐    ┌──
                          └────┘    └────┘

Interrupts:           ↑  ↑  ↑  ↑  ↑  ↑  ↑  ↑
                      (Each edge triggers ISR)

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
- **Low Latency**: Interrupts handled on dedicated core
- **Thread Safety**: Atomic operations and mutexes prevent race conditions
- **Responsiveness**: Web server doesn't block encoder processing
- **Reliability**: Each core can operate independently
- **Scalability**: Easy to add more endpoints or encoder features
