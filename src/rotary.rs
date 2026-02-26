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
    pub num_target_angles: u8,
    pub tick_size_multiplier: f32,
    pub number_of_runs: u32,
    pub update_rate_ms: u32,
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
            step_mode: StepMode::Full,
            output_pin: 32,
            output_default_state: PinState::Low,
            minimum_angle_threshold: 2.5,
            hold_output_until_threshold: false,
            debug_enabled: false,
            num_target_angles: 1,
            tick_size_multiplier: 2.0,
            number_of_runs: 1,
            update_rate_ms: 200,
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
    pub current_run: Arc<AtomicI32>,
    pub total_runs: Arc<AtomicI32>,
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
            current_run: Arc::new(AtomicI32::new(0)),
            total_runs: Arc::new(AtomicI32::new(1)),
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
        let number_of_runs = settings.number_of_runs;
        drop(settings);
        
        let mut targets = self.target_angles.lock()
            .expect("Target angles mutex poisoned");
        targets.clear();
        // Convert degrees to steps, with validation
        for angle in angles {
            // Clamp angles to valid range [0, 360]
            let clamped_angle = angle.max(0.0).min(360.0);
            targets.push((clamped_angle * multiplier).round() as i32);
        }
        *self.current_target_index.lock()
            .expect("Current target index mutex poisoned") = 0;
        self.triggered.store(false, Ordering::SeqCst);
        self.reset_detected.store(false, Ordering::SeqCst);
        self.encoder_active.store(true, Ordering::SeqCst);
        // Reset angle to 0 when Start button is pressed
        self.set_value(0);
        // Initialize run counters
        self.reset_current_run();
        self.set_total_runs(number_of_runs as i32);
        self.increment_current_run(); // Start at run 1
    }

    pub fn stop(&self) {
        self.encoder_active.store(false, Ordering::SeqCst);
        self.output_on.store(false, Ordering::SeqCst);
        // Reset angle to 0 when Stop button is pressed
        self.set_value(0);
        // Reset run counter when stopping
        self.reset_current_run();
        // Stop has highest priority - clear any manual output override
        self.clear_manual_output();
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

    pub fn get_current_run(&self) -> i32 {
        self.current_run.load(Ordering::SeqCst)
    }

    pub fn get_total_runs(&self) -> i32 {
        self.total_runs.load(Ordering::SeqCst)
    }

    pub fn set_total_runs(&self, runs: i32) {
        self.total_runs.store(runs, Ordering::SeqCst);
    }

    pub fn increment_current_run(&self) {
        self.current_run.fetch_add(1, Ordering::SeqCst);
    }

    pub fn reset_current_run(&self) {
        self.current_run.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state_with_step_mode(mode: StepMode) -> RotaryEncoderState {
        let state = RotaryEncoderState::new(0, 720);
        let mut settings = Settings::default();
        settings.step_mode = mode;
        state.set_settings(settings);
        state
    }

    // --- StepMode default ---

    #[test]
    fn default_step_mode_is_full() {
        let settings = Settings::default();
        assert_eq!(settings.step_mode, StepMode::Full);
    }

    // --- set_target_angles: rounding instead of truncation ---

    #[test]
    fn target_angle_half_degree_full_mode_rounds_to_one_step() {
        // With Full mode (multiplier=1.0), 0.5° should round to 1 step, not truncate to 0.
        // Previously `(0.5 * 1.0) as i32 = 0` caused immediate trigger (critical bug).
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![0.5]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 1, "0.5° in Full mode must round to 1 step, not truncate to 0");
    }

    #[test]
    fn target_angle_zero_not_set_for_half_degree_full_mode() {
        // Ensure the target is never 0 for a 0.5° input in Full mode (prevents immediate trigger).
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![0.5]);
        let targets = state.target_angles.lock().unwrap();
        assert_ne!(targets[0], 0, "Target of 0 steps would trigger immediately at start");
    }

    #[test]
    fn target_angle_one_degree_full_mode_is_one_step() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![1.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 1);
    }

    #[test]
    fn target_angle_half_degree_half_mode_is_one_step() {
        // With Half mode (multiplier=2.0), 0.5° = (0.5 * 2.0).round() = 1 step.
        let state = make_state_with_step_mode(StepMode::Half);
        state.set_target_angles(vec![0.5]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 1);
    }

    #[test]
    fn target_angle_one_degree_half_mode_is_two_steps() {
        let state = make_state_with_step_mode(StepMode::Half);
        state.set_target_angles(vec![1.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 2);
    }

    #[test]
    fn target_angle_45_degrees_full_mode() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![45.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 45);
    }

    #[test]
    fn target_angle_45_degrees_half_mode() {
        let state = make_state_with_step_mode(StepMode::Half);
        state.set_target_angles(vec![45.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 90);
    }

    // --- get_angle: correct degree conversion ---

    #[test]
    fn get_angle_full_mode_one_step_is_one_degree() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_value(1);
        assert!((state.get_angle() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn get_angle_half_mode_two_steps_is_one_degree() {
        let state = make_state_with_step_mode(StepMode::Half);
        state.set_value(2);
        assert!((state.get_angle() - 1.0).abs() < 1e-6);
    }

    // --- get_target_angles: round-trip conversion ---

    #[test]
    fn get_target_angles_round_trip_full_mode() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![45.0, 90.0, 180.0]);
        let retrieved = state.get_target_angles();
        assert_eq!(retrieved, vec![45.0, 90.0, 180.0]);
    }

    #[test]
    fn get_target_angles_round_trip_half_mode() {
        let state = make_state_with_step_mode(StepMode::Half);
        state.set_target_angles(vec![45.0, 90.0]);
        let retrieved = state.get_target_angles();
        assert_eq!(retrieved, vec![45.0, 90.0]);
    }

    // --- angle clamping ---

    #[test]
    fn target_angle_negative_clamped_to_zero() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![-10.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 0);
    }

    #[test]
    fn target_angle_above_360_clamped_to_360() {
        let state = make_state_with_step_mode(StepMode::Full);
        state.set_target_angles(vec![400.0]);
        let targets = state.target_angles.lock().unwrap();
        assert_eq!(targets[0], 360);
    }

    // --- update_from_direction ---

    #[test]
    fn update_from_direction_clockwise_increments() {
        let state = RotaryEncoderState::new(0, 720);
        state.update_from_direction(1);
        assert_eq!(state.get_value(), 1);
    }

    #[test]
    fn update_from_direction_anticlockwise_decrements_clamped_at_min() {
        let state = RotaryEncoderState::new(0, 720);
        state.update_from_direction(-1);
        // Clamped at min_val=0
        assert_eq!(state.get_value(), 0);
    }

    // --- stop: highest priority ---

    #[test]
    fn stop_deactivates_output() {
        let state = RotaryEncoderState::new(0, 720);
        state.output_on.store(true, Ordering::SeqCst);
        state.stop();
        assert!(!state.is_output_on(), "stop() must deactivate the output (set output_on to false)");
    }

    #[test]
    fn stop_clears_manual_output_override() {
        let state = RotaryEncoderState::new(0, 720);
        state.set_manual_output(true);
        assert!(state.is_manual_output_override());
        state.stop();
        assert!(
            !state.is_manual_output_override(),
            "stop() must clear manual output override (Stop has highest priority)"
        );
    }

    #[test]
    fn stop_clears_manual_output_override_when_output_off() {
        let state = RotaryEncoderState::new(0, 720);
        state.set_manual_output(false);
        assert!(state.is_manual_output_override());
        state.stop();
        assert!(
            !state.is_manual_output_override(),
            "stop() must clear manual output override regardless of manual state"
        );
    }

    #[test]
    fn stop_deactivates_encoder() {
        let state = RotaryEncoderState::new(0, 720);
        state.set_target_angles(vec![45.0]);
        assert!(state.is_active());
        state.stop();
        assert!(!state.is_active(), "stop() must deactivate the encoder");
    }
}
