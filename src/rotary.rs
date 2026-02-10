use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub forward_direction: ForwardDirection,
    pub step_mode: StepMode,
    pub output_pin: u8,
    pub output_default_state: PinState,
    pub minimum_angle_threshold: f32,
    pub hold_output_until_threshold: bool,
    pub debug_enabled: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ForwardDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum StepMode {
    Full,  // 1 degree per step
    Half,  // 0.5 degrees per step
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum PinState {
    Low,
    High,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            forward_direction: ForwardDirection::Clockwise,
            step_mode: StepMode::Half,
            output_pin: 32,
            output_default_state: PinState::Low,
            minimum_angle_threshold: 2.5,
            hold_output_until_threshold: false,
            debug_enabled: false,
        }
    }
}

#[derive(Clone)]
pub struct RotaryEncoderState {
    pub value: Arc<AtomicI32>,
    pub target_angles: Arc<Mutex<Vec<i32>>>,
    pub current_target_index: Arc<Mutex<usize>>,
    pub encoder_active: Arc<AtomicBool>,
    pub output_on: Arc<AtomicBool>,
    pub triggered: Arc<AtomicBool>,
    pub reset_detected: Arc<AtomicBool>,
    min_val: i32,
    max_val: i32,
    pub debug_mode: Arc<AtomicBool>,
    pub settings: Arc<Mutex<Settings>>,
    pub manual_output_override: Arc<AtomicBool>,
    pub manual_output_state: Arc<AtomicBool>,
}

impl RotaryEncoderState {
    pub fn new(min_val: i32, max_val: i32) -> Self {
        Self {
            value: Arc::new(AtomicI32::new(min_val)),
            target_angles: Arc::new(Mutex::new(Vec::new())),
            current_target_index: Arc::new(Mutex::new(0)),
            encoder_active: Arc::new(AtomicBool::new(false)),
            output_on: Arc::new(AtomicBool::new(false)),
            triggered: Arc::new(AtomicBool::new(false)),
            reset_detected: Arc::new(AtomicBool::new(false)),
            min_val,
            max_val,
            debug_mode: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(Mutex::new(Settings::default())),
            manual_output_override: Arc::new(AtomicBool::new(false)),
            manual_output_state: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_value(&self, val: i32) {
        self.value.store(val, Ordering::SeqCst);
    }

    pub fn get_value(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn get_angle(&self) -> f32 {
        let divisor = {
            let settings = self.settings.lock().expect("Settings mutex poisoned");
            match settings.step_mode {
                StepMode::Full => 1.0,
                StepMode::Half => 2.0,
            }
        };
        self.get_value() as f32 / divisor
    }

    pub fn is_active(&self) -> bool {
        self.encoder_active.load(Ordering::SeqCst)
    }

    pub fn is_output_on(&self) -> bool {
        self.output_on.load(Ordering::SeqCst)
    }

    pub fn set_target_angles(&self, angles: Vec<f32>) {
        let settings = self.settings.lock().expect("Settings mutex poisoned");
        let multiplier = match settings.step_mode {
            StepMode::Full => 1.0,
            StepMode::Half => 2.0,
        };
        drop(settings);
        
        let mut targets = self.target_angles.lock()
            .expect("Target angles mutex poisoned");
        targets.clear();
        // Convert degrees to steps, with validation
        for angle in angles {
            // Clamp angles to valid range [0, 360]
            let clamped_angle = angle.max(0.0).min(360.0);
            targets.push((clamped_angle * multiplier) as i32);
        }
        *self.current_target_index.lock()
            .expect("Current target index mutex poisoned") = 0;
        self.triggered.store(false, Ordering::SeqCst);
        self.reset_detected.store(false, Ordering::SeqCst);
        self.encoder_active.store(true, Ordering::SeqCst);
        // Reset angle to 0 when Start button is pressed
        self.set_value(0);
    }

    pub fn stop(&self) {
        self.encoder_active.store(false, Ordering::SeqCst);
        self.output_on.store(false, Ordering::SeqCst);
        // Reset angle to 0 when Stop button is pressed
        self.set_value(0);
    }

    pub fn get_target_angles(&self) -> Vec<f32> {
        let settings = self.settings.lock().expect("Settings mutex poisoned");
        let divisor = match settings.step_mode {
            StepMode::Full => 1.0,
            StepMode::Half => 2.0,
        };
        drop(settings);
        
        self.target_angles
            .lock()
            .expect("Target angles mutex poisoned")
            .iter()
            .map(|&v| v as f32 / divisor)
            .collect()
    }

    pub fn get_current_target_index(&self) -> usize {
        *self.current_target_index.lock()
            .expect("Current target index mutex poisoned")
    }

    pub fn set_debug_mode(&self, enabled: bool) {
        self.debug_mode.store(enabled, Ordering::Release);
    }

    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode.load(Ordering::Acquire)
    }

    pub fn is_target_reached(&self) -> bool {
        self.triggered.load(Ordering::Acquire)
    }

    fn bound(&self, value: i32) -> i32 {
        if value < self.min_val {
            self.min_val
        } else if value > self.max_val {
            self.max_val
        } else {
            value
        }
    }

    // Update encoder value based on direction from rotary-encoder-embedded library
    pub fn update_from_direction(&self, direction: i32) {
        if direction != 0 {
            let settings = self.settings.lock().expect("Settings mutex poisoned");
            let forward_direction = settings.forward_direction;
            drop(settings);
            
            let old_value = self.get_value();
            // Apply direction based on forward_direction setting
            let adjusted_direction = match forward_direction {
                ForwardDirection::Clockwise => direction,
                ForwardDirection::CounterClockwise => -direction,
            };
            let new_value = self.bound(old_value + adjusted_direction);
            self.value.store(new_value, Ordering::SeqCst);
            
            if self.is_debug_mode() {
                let angle = self.get_angle();
                log::info!("🔍 DEBUG: Direction={} Value={} Angle={:.1}°", adjusted_direction, new_value, angle);
            }
        }
    }

    pub fn get_settings(&self) -> Settings {
        let mut settings = self.settings.lock().expect("Settings mutex poisoned").clone();
        // Sync debug_enabled with the atomic debug_mode
        settings.debug_enabled = self.is_debug_mode();
        settings
    }

    pub fn set_settings(&self, new_settings: Settings) {
        // Sync the atomic debug_mode with debug_enabled from settings
        self.set_debug_mode(new_settings.debug_enabled);
        let mut settings = self.settings.lock().expect("Settings mutex poisoned");
        *settings = new_settings;
    }

    pub fn set_manual_output(&self, state: bool) {
        self.manual_output_override.store(true, Ordering::SeqCst);
        self.manual_output_state.store(state, Ordering::SeqCst);
    }

    pub fn clear_manual_output(&self) {
        self.manual_output_override.store(false, Ordering::SeqCst);
    }

    pub fn is_manual_output_override(&self) -> bool {
        self.manual_output_override.load(Ordering::SeqCst)
    }

    pub fn get_manual_output_state(&self) -> bool {
        self.manual_output_state.load(Ordering::SeqCst)
    }
}
