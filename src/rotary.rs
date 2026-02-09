use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

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
        }
    }

    pub fn set_value(&self, val: i32) {
        self.value.store(val, Ordering::SeqCst);
    }

    pub fn get_value(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn get_angle(&self) -> f32 {
        self.get_value() as f32 / 2.0
    }

    pub fn is_active(&self) -> bool {
        self.encoder_active.load(Ordering::SeqCst)
    }

    pub fn is_output_on(&self) -> bool {
        self.output_on.load(Ordering::SeqCst)
    }

    pub fn set_target_angles(&self, angles: Vec<f32>) {
        let mut targets = self.target_angles.lock()
            .expect("Target angles mutex poisoned");
        targets.clear();
        // Convert degrees to half-steps, with validation
        for angle in angles {
            // Clamp angles to valid range [0, 360]
            let clamped_angle = angle.max(0.0).min(360.0);
            targets.push((clamped_angle * 2.0) as i32);
        }
        *self.current_target_index.lock()
            .expect("Current target index mutex poisoned") = 0;
        self.triggered.store(false, Ordering::SeqCst);
        self.reset_detected.store(false, Ordering::SeqCst);
        self.encoder_active.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.encoder_active.store(false, Ordering::SeqCst);
        self.output_on.store(false, Ordering::SeqCst);
    }

    pub fn get_target_angles(&self) -> Vec<f32> {
        self.target_angles
            .lock()
            .expect("Target angles mutex poisoned")
            .iter()
            .map(|&v| v as f32 / 2.0)
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
            let old_value = self.get_value();
            let new_value = self.bound(old_value + direction);
            self.value.store(new_value, Ordering::SeqCst);
            
            if self.is_debug_mode() {
                let angle = new_value as f32 / 2.0;
                log::info!("🔍 DEBUG: Direction={} Value={} Angle={:.1}°", direction, new_value, angle);
            }
        }
    }
}
