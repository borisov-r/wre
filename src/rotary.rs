use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

// Rotary encoder states for half-step operation
const R_START: u8 = 0x0;
const R_CW_1: u8 = 0x1;
const R_CW_2: u8 = 0x2;
const R_CW_3: u8 = 0x3;
const R_CCW_1: u8 = 0x4;
const R_CCW_2: u8 = 0x5;

const DIR_CW: u8 = 0x10;
const DIR_CCW: u8 = 0x20;
const STATE_MASK: u8 = 0x07;
const DIR_MASK: u8 = 0x30;

// Half-step transition table
const TRANSITION_TABLE_HALF_STEP: [[u8; 4]; 8] = [
    [R_CW_3, R_CW_2, R_CW_1, R_START],
    [R_CW_3 | DIR_CCW, R_START, R_CW_1, R_START],
    [R_CW_3 | DIR_CW, R_CW_2, R_START, R_START],
    [R_CW_3, R_CCW_2, R_CCW_1, R_START],
    [R_CW_3, R_CW_2, R_CCW_1, R_START | DIR_CW],
    [R_CW_3, R_CCW_2, R_CW_3, R_START | DIR_CCW],
    [R_START, R_START, R_START, R_START],
    [R_START, R_START, R_START, R_START],
];

#[derive(Clone)]
pub struct RotaryEncoderState {
    pub value: Arc<AtomicI32>,
    pub target_angles: Arc<Mutex<Vec<i32>>>,
    pub current_target_index: Arc<Mutex<usize>>,
    pub encoder_active: Arc<AtomicBool>,
    pub output_on: Arc<AtomicBool>,
    pub triggered: Arc<AtomicBool>,
    pub reset_detected: Arc<AtomicBool>,
    state: Arc<Mutex<u8>>,
    min_val: i32,
    max_val: i32,
    reverse: bool,
}

impl RotaryEncoderState {
    pub fn new(min_val: i32, max_val: i32, reverse: bool) -> Self {
        Self {
            value: Arc::new(AtomicI32::new(min_val)),
            target_angles: Arc::new(Mutex::new(Vec::new())),
            current_target_index: Arc::new(Mutex::new(0)),
            encoder_active: Arc::new(AtomicBool::new(false)),
            output_on: Arc::new(AtomicBool::new(false)),
            triggered: Arc::new(AtomicBool::new(false)),
            reset_detected: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(R_START)),
            min_val,
            max_val,
            reverse,
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

    fn bound(&self, value: i32) -> i32 {
        if value < self.min_val {
            self.min_val
        } else if value > self.max_val {
            self.max_val
        } else {
            value
        }
    }

    // Process rotary encoder pin changes
    // Note: This is called from ISR context. The mutex is held briefly (~1μs)
    // during state transition. For even better performance, this could be
    // reimplemented using atomic state machine or lock-free algorithm.
    pub fn process_pins(&self, clk_value: bool, dt_value: bool) {
        let old_value = self.get_value();
        let clk_dt_pins = ((clk_value as u8) << 1) | (dt_value as u8);

        let mut state = self.state.lock()
            .expect("State machine mutex poisoned");
        *state = TRANSITION_TABLE_HALF_STEP[(*state & STATE_MASK) as usize][clk_dt_pins as usize];
        let direction = *state & DIR_MASK;

        let mut incr = 0;
        if direction == DIR_CW {
            incr = 1;
        } else if direction == DIR_CCW {
            incr = -1;
        }

        if self.reverse {
            incr = -incr;
        }

        let new_value = self.bound(old_value + incr);
        self.value.store(new_value, Ordering::SeqCst);
    }
}
