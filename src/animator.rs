use crate::engine::patterns::PhaseStyle;

pub const COLOR_INHALE: (f64, f64, f64) = (0.0, 255.0, 255.0);
pub const COLOR_HOLD:   (f64, f64, f64) = (255.0, 230.0, 0.0);
pub const COLOR_EXHALE: (f64, f64, f64) = (0.0, 220.0, 100.0);

pub fn phase_color(style: &PhaseStyle) -> (f64, f64, f64) {
    match style {
        PhaseStyle::Rising  => COLOR_INHALE,
        PhaseStyle::Steady  => COLOR_HOLD,
        PhaseStyle::Falling => COLOR_EXHALE,
    }
}

pub fn cubic_in_out(t: f64) -> f64 {
    if t < 0.5 { 4.0 * t * t * t }
    else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

fn quad_in_out(t: f64) -> f64 {
    if t < 0.5 { 2.0 * t * t }
    else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

/// Smoothly interpolates toward a target value over `duration` seconds.
pub struct Lerp {
    current:  f64,
    start:    f64,
    target:   f64,
    elapsed:  f64,
    duration: f64,
}

impl Lerp {
    pub fn new(value: f64, duration: f64) -> Self {
        Self { current: value, start: value, target: value, elapsed: duration, duration }
    }

    pub fn set(&mut self, target: f64) {
        self.start   = self.current;
        self.target  = target;
        self.elapsed = 0.0;
    }

    fn tick(&mut self, delta: f64) {
        if self.elapsed < self.duration {
            self.elapsed = (self.elapsed + delta).min(self.duration);
            let t = cubic_in_out(self.elapsed / self.duration);
            self.current = self.start + (self.target - self.start) * t;
        }
    }
}

impl std::ops::Deref for Lerp {
    type Target = f64;
    fn deref(&self) -> &f64 { &self.current }
}

/// Reveals a string character-by-character over `duration` seconds.
pub struct Typewriter {
    full:     String,
    current:  String,
    elapsed:  f64,
    duration: f64,
}

impl Typewriter {
    pub fn new(text: String, duration: f64) -> Self {
        let current = text.clone();
        Self { full: text, current, elapsed: duration, duration }
    }

    pub fn set(&mut self, text: String) {
        self.full    = text;
        self.current = String::new();
        self.elapsed = 0.0;
    }

    pub fn get(&self) -> &str { &self.current }

    fn tick(&mut self, delta: f64) {
        if self.elapsed < self.duration {
            self.elapsed = (self.elapsed + delta).min(self.duration);
            let total   = self.full.chars().count();
            let visible = ((self.elapsed / self.duration) * total as f64).ceil() as usize;
            self.current = self.full.chars().take(visible).collect();
        }
    }
}

/// Oscillates between `min` and 1.0 with a smooth quad_in_out curve.
pub struct Pulse {
    min:     f64,
    max:     f64,
    period:  f64,
    elapsed: f64,
    current: f64,
}

impl Pulse {
    pub fn new(value: f64, period: f64) -> Self {
        Self { min: value, max: value, period, elapsed: 0.0, current: value }
    }

    pub fn set(&mut self, min: f64) {
        self.min     = min;
        self.max     = 1.0;
        self.elapsed = 0.0;
    }

    fn tick(&mut self, delta: f64) {
        if (self.min - self.max).abs() < f64::EPSILON { return; }
        self.elapsed = (self.elapsed + delta) % self.period;
        let half = self.period / 2.0;
        let t = if self.elapsed < half {
            quad_in_out(self.elapsed / half)
        } else {
            quad_in_out(1.0 - (self.elapsed - half) / half)
        };
        self.current = self.max - (self.max - self.min) * t;
    }
}

impl std::ops::Deref for Pulse {
    type Target = f64;
    fn deref(&self) -> &f64 { &self.current }
}

pub struct SessionAnimator {
    pub color_r:     Lerp,
    pub color_g:     Lerp,
    pub color_b:     Lerp,
    pub phase_label: Typewriter,
    pub hold_pulse:  Pulse,
}

impl SessionAnimator {
    pub fn for_phase(style: &PhaseStyle, label: &str) -> Self {
        let (r, g, b) = phase_color(style);
        let mut anim = Self {
            color_r:     Lerp::new(r, 0.8),
            color_g:     Lerp::new(g, 0.8),
            color_b:     Lerp::new(b, 0.8),
            phase_label: Typewriter::new(label.to_string(), 0.5),
            hold_pulse:  Pulse::new(1.0, 0.9),
        };
        anim.hold_pulse.set(0.65);
        anim
    }

    pub fn tick(&mut self, delta: f64) {
        self.color_r.tick(delta);
        self.color_g.tick(delta);
        self.color_b.tick(delta);
        self.phase_label.tick(delta);
        self.hold_pulse.tick(delta);
    }
}
