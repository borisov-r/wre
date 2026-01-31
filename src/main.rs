mod rotary;
mod webserver;

use esp_idf_hal::gpio::{Gpio21, Gpio22, Gpio32, PinDriver, Pull, InterruptType};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_sys as _;
use log::*;
use rotary::RotaryEncoderState;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF services
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("🔧 ESP32 Rotary Encoder Control - Rust Edition");
    info!("Starting dual-core application...");

    let peripherals = Peripherals::take()?;

    // Create rotary encoder state (0-720 half-steps = 0-360 degrees)
    let encoder_state = RotaryEncoderState::new(0, 720, true);
    let encoder_state_clone = encoder_state.clone();
    let encoder_state_web = encoder_state.clone();

    // Set up GPIO pins for rotary encoder (CLK=21, DT=22)
    let clk_pin = peripherals.pins.gpio21;
    let dt_pin = peripherals.pins.gpio22;
    let output_pin = peripherals.pins.gpio32;

    // Spawn rotary encoder task on Core 1 (dedicated for interrupts and encoder)
    info!("Starting rotary encoder task on Core 1...");
    ThreadSpawnConfiguration {
        name: Some(b"rotary_core\0"),
        stack_size: 8192,
        priority: 5,
        pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
        ..Default::default()
    }
    .set()?;

    thread::Builder::new()
        .stack_size(8192)
        .name("rotary_core".to_string())
        .spawn(move || {
            if let Err(e) = rotary_task(encoder_state_clone, clk_pin, dt_pin, output_pin) {
                error!("Rotary task error: {:?}", e);
            }
        })?;

    // Run web server on Core 0 (main core for networking)
    info!("Starting web server on Core 0...");
    ThreadSpawnConfiguration {
        name: Some(b"webserver_core\0"),
        stack_size: 16384,
        priority: 5,
        pin_to_core: Some(esp_idf_hal::cpu::Core::Core0),
        ..Default::default()
    }
    .set()?;

    // Start webserver (blocks on this core)
    webserver::start_webserver(encoder_state_web, peripherals.modem)?;

    Ok(())
}

fn rotary_task(
    encoder_state: RotaryEncoderState,
    clk_pin: Gpio21,
    dt_pin: Gpio22,
    output_pin: Gpio32,
) -> anyhow::Result<()> {
    info!("Rotary encoder task running on Core 1");

    // Set up input pins with pull-up resistors
    let mut clk = PinDriver::input(clk_pin)?;
    clk.set_pull(Pull::Up)?;
    clk.set_interrupt_type(InterruptType::AnyEdge)?;

    let mut dt = PinDriver::input(dt_pin)?;
    dt.set_pull(Pull::Up)?;
    dt.set_interrupt_type(InterruptType::AnyEdge)?;

    // Pin numbers for low-level GPIO operations
    let clk_pin_num = 21;
    let dt_pin_num = 22;

    // Additional low-level GPIO configuration to ensure pull-ups are enabled
    // This is a belt-and-suspenders approach to ensure GPIO is configured correctly
    unsafe {
        // Set GPIO direction to input
        esp_idf_sys::gpio_set_direction(clk_pin_num, esp_idf_sys::gpio_mode_t_GPIO_MODE_INPUT);
        esp_idf_sys::gpio_set_direction(dt_pin_num, esp_idf_sys::gpio_mode_t_GPIO_MODE_INPUT);
        
        // Explicitly enable pull-up resistors
        esp_idf_sys::gpio_set_pull_mode(clk_pin_num, esp_idf_sys::gpio_pull_mode_t_GPIO_PULLUP_ONLY);
        esp_idf_sys::gpio_set_pull_mode(dt_pin_num, esp_idf_sys::gpio_pull_mode_t_GPIO_PULLUP_ONLY);
        
        info!("✓ GPIO pins explicitly configured as INPUT with PULL-UP");
    }

    // Verify pin configuration by reading initial states
    // With pull-up resistors, pins should read HIGH (true) when not connected or encoder is idle
    let clk_initial = clk.is_high();
    let dt_initial = dt.is_high();
    info!("📌 Pin configuration verified - CLK initial state: {} ({}), DT initial state: {} ({})", 
          if clk_initial { "HIGH" } else { "LOW" },
          if clk_initial { "1" } else { "0" },
          if dt_initial { "HIGH" } else { "LOW" },
          if dt_initial { "1" } else { "0" });

    // Set up output pin
    let mut output = PinDriver::output(output_pin)?;
    output.set_low()?;

    // Create shared state for ISR
    let encoder_state_isr = encoder_state.clone();

    // Set up interrupt handlers
    // IMPORTANT: Must keep subscription handles alive, otherwise interrupts are unregistered
    let _clk_subscription;
    let _dt_subscription;
    
    unsafe {
        _clk_subscription = clk.subscribe({
            let encoder_state = encoder_state_isr.clone();
            let clk_num = clk_pin_num;  // Explicitly capture for closure
            let dt_num = dt_pin_num;    // Explicitly capture for closure
            
            move || {
                // Read both pin states
                let clk_val = esp_idf_sys::gpio_get_level(clk_num) != 0;
                let dt_val = esp_idf_sys::gpio_get_level(dt_num) != 0;
                encoder_state.process_pins(clk_val, dt_val);
            }
        })?;

        _dt_subscription = dt.subscribe({
            let encoder_state = encoder_state_isr.clone();
            let clk_num = clk_pin_num;  // Explicitly capture for closure
            let dt_num = dt_pin_num;    // Explicitly capture for closure
            
            move || {
                // Read both pin states
                let clk_val = esp_idf_sys::gpio_get_level(clk_num) != 0;
                let dt_val = esp_idf_sys::gpio_get_level(dt_num) != 0;
                encoder_state.process_pins(clk_val, dt_val);
            }
        })?;
    }
    
    info!("✓ Interrupt handlers subscribed for GPIO {} (CLK) and GPIO {} (DT)", clk_pin_num, dt_pin_num);

    // Main rotary encoder loop
    loop {
        // In debug mode, always read and display current pin states even when not active
        // This helps diagnose if pins are responding to encoder rotation
        if encoder_state.is_debug_mode() {
            // Read pin states directly using low-level GPIO call
            let clk_current = unsafe { esp_idf_sys::gpio_get_level(clk_pin_num) != 0 };
            let dt_current = unsafe { esp_idf_sys::gpio_get_level(dt_pin_num) != 0 };
            let (isr_clk, isr_dt, state, value, debug_angle, isr_count, clk_dt_pins) = encoder_state.get_debug_info();
            
            // Log pin states continuously when in debug mode to help diagnose issues
            // Show both live-read pins and ISR-captured pins for comparison
            info!("🔍 DEBUG: Live[CLK={} DT={}] ISR[CLK={} DT={} Pins=0b{:02b}] State=0x{:02X} Value={} Angle={:.1}° ISR_Calls={}", 
                  if clk_current { 1 } else { 0 },
                  if dt_current { 1 } else { 0 },
                  if isr_clk { 1 } else { 0 },
                  if isr_dt { 1 } else { 0 },
                  clk_dt_pins,
                  state,
                  value,
                  debug_angle,
                  isr_count);
        }
        
        if !encoder_state.is_active() {
            thread::sleep(Duration::from_millis(200));
            continue;
        }

        let targets = encoder_state.target_angles.lock()
            .expect("Target angles mutex poisoned");
        if targets.is_empty() {
            drop(targets);
            thread::sleep(Duration::from_millis(200));
            continue;
        }

        let current_idx = *encoder_state.current_target_index.lock()
            .expect("Current target index mutex poisoned");
        if current_idx >= targets.len() {
            drop(targets);
            thread::sleep(Duration::from_millis(200));
            continue;
        }

        let target = targets[current_idx];
        drop(targets);

        let steps = encoder_state.get_value();
        let angle = steps as f32 / 2.0;
        let target_angle = target as f32 / 2.0;

        // Print debug information to serial port when debug mode is enabled
        if encoder_state.is_debug_mode() {
            let (isr_clk, isr_dt, state, value, debug_angle, isr_count, clk_dt_pins) = encoder_state.get_debug_info();
            info!("🔍 DEBUG (Active): ISR[CLK={} DT={} Pins=0b{:02b}] State=0x{:02X} Value={} Angle={:.1}° Target={:.1}° ISR_Calls={}", 
                  if isr_clk { 1 } else { 0 },
                  if isr_dt { 1 } else { 0 },
                  clk_dt_pins,
                  state,
                  value,
                  debug_angle,
                  target_angle,
                  isr_count);
        }

        // Trigger output when reaching target (moving forward from 0)
        if !encoder_state.triggered.load(std::sync::atomic::Ordering::SeqCst) 
            && steps >= target {
            output.set_high()?;
            encoder_state.output_on.store(true, std::sync::atomic::Ordering::SeqCst);
            encoder_state.triggered.store(true, std::sync::atomic::Ordering::SeqCst);
            info!("⚡ Target reached: {:.1}°", target_angle);
        } else if encoder_state.triggered.load(std::sync::atomic::Ordering::SeqCst) {
            // Keep output on while above target
            if steps < target {
                output.set_low()?;
                encoder_state.output_on.store(false, std::sync::atomic::Ordering::SeqCst);
            }
        } else {
            output.set_low()?;
            encoder_state.output_on.store(false, std::sync::atomic::Ordering::SeqCst);
        }

        // Reset encoder if angle drops below 2°
        if angle < 2.0 && !encoder_state.reset_detected.load(std::sync::atomic::Ordering::SeqCst) {
            encoder_state.set_value(0);
            encoder_state.reset_detected.store(true, std::sync::atomic::Ordering::SeqCst);
            encoder_state.triggered.store(false, std::sync::atomic::Ordering::SeqCst);
            info!("🔄 Encoder reset to 0°");

            // Advance to next target
            let mut idx = encoder_state.current_target_index.lock()
                .expect("Current target index mutex poisoned");
            *idx += 1;
            let new_idx = *idx;
            drop(idx);

            let targets = encoder_state.target_angles.lock()
                .expect("Target angles mutex poisoned");
            if new_idx >= targets.len() {
                info!("✅ All targets completed and returned to 0°.");
                encoder_state.stop();
                output.set_low()?;
            }
            drop(targets);
        }

        if angle > 5.0 {
            encoder_state.reset_detected.store(false, std::sync::atomic::Ordering::SeqCst);
        }

        thread::sleep(Duration::from_millis(50));
    }
}
