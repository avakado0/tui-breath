use super::patterns::{Pattern, PhaseStyle};

#[derive(Clone, Copy, Debug)]
pub struct BreathingEngine {
    pub pattern: &'static Pattern,
    pub tempo: f64,
    pub current_phase_idx: usize,
    pub phase_elapsed_secs: f64,
    pub cycle_count: u32,
    pub total_elapsed_secs: f64,
    pub duration_target_secs: f64,
    pub is_paused: bool,
    pub pause_count: u32,
}

impl BreathingEngine {
    pub fn new(
        pattern: &'static Pattern,
        tempo: f64,
        duration_target_secs: f64,
    ) -> Self {
        Self {
            pattern,
            tempo: tempo.max(0.1).min(2.0), // Clamp tempo to reasonable range
            current_phase_idx: 0,
            phase_elapsed_secs: 0.0,
            cycle_count: 0,
            total_elapsed_secs: 0.0,
            duration_target_secs,
            is_paused: false,
            pause_count: 0,
        }
    }

    pub fn current_phase(&self) -> &'static super::patterns::Phase {
        &self.pattern.phases[self.current_phase_idx]
    }

    pub fn current_phase_duration(&self) -> f64 {
        self.current_phase().duration_secs / self.tempo
    }

    pub fn phase_progress(&self) -> f64 {
        let duration = self.current_phase_duration();
        (self.phase_elapsed_secs / duration).min(1.0)
    }

    pub fn phase_remaining(&self) -> f64 {
        (self.current_phase_duration() - self.phase_elapsed_secs).max(0.0)
    }

    pub fn is_complete(&self) -> bool {
        self.total_elapsed_secs >= self.duration_target_secs
    }

    pub fn completion_percent(&self) -> f64 {
        (self.total_elapsed_secs / self.duration_target_secs * 100.0).min(100.0)
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        if self.is_paused {
            self.pause_count += 1;
        }
    }

    fn advance_phase(&mut self) {
        self.current_phase_idx += 1;
        if self.current_phase_idx >= self.pattern.phases.len() {
            self.current_phase_idx = 0;
            self.cycle_count += 1;
        }
        self.phase_elapsed_secs = 0.0;
    }

    pub fn tick(&mut self, delta_secs: f64) {
        if self.is_paused || self.is_complete() {
            return;
        }

        self.phase_elapsed_secs += delta_secs;
        self.total_elapsed_secs += delta_secs;

        if self.phase_elapsed_secs >= self.current_phase_duration() {
            self.advance_phase();
        }

        // Cap at target
        if self.total_elapsed_secs > self.duration_target_secs {
            self.total_elapsed_secs = self.duration_target_secs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::patterns::PATTERN_BOX;

    #[test]
    fn test_phase_progression() {
        let mut engine = BreathingEngine::new(&PATTERN_BOX, 1.0, 60.0);

        assert_eq!(engine.current_phase_idx, 0);
        assert_eq!(engine.cycle_count, 0);

        // Advance through one phase (4 seconds)
        engine.tick(4.0);
        assert_eq!(engine.current_phase_idx, 1); // Should advance to next phase
    }

    #[test]
    fn test_tempo_affects_duration() {
        let mut engine = BreathingEngine::new(&PATTERN_BOX, 2.0, 60.0);
        let phase_duration = engine.current_phase_duration();

        // At tempo 2.0, 4 second phase becomes 2 seconds
        assert!((phase_duration - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_completion() {
        let mut engine = BreathingEngine::new(&PATTERN_BOX, 1.0, 10.0);

        assert!(!engine.is_complete());
        engine.tick(10.0);
        assert!(engine.is_complete());
    }

    #[test]
    fn test_pause() {
        let mut engine = BreathingEngine::new(&PATTERN_BOX, 1.0, 60.0);

        engine.toggle_pause();
        assert!(engine.is_paused);
        assert_eq!(engine.pause_count, 1);

        let elapsed_before = engine.total_elapsed_secs;
        engine.tick(1.0);
        let elapsed_after = engine.total_elapsed_secs;

        // Time should not advance while paused
        assert_eq!(elapsed_before, elapsed_after);
    }
}
