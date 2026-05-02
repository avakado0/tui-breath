use animate::animate;

use crate::engine::patterns::PhaseStyle;

pub const COLOR_INHALE: (f64, f64, f64) = (0.0, 255.0, 255.0);
pub const COLOR_HOLD: (f64, f64, f64) = (255.0, 230.0, 0.0);
pub const COLOR_EXHALE: (f64, f64, f64) = (0.0, 220.0, 100.0);

pub fn phase_color(style: &PhaseStyle) -> (f64, f64, f64) {
    match style {
        PhaseStyle::Rising => COLOR_INHALE,
        PhaseStyle::Steady => COLOR_HOLD,
        PhaseStyle::Falling => COLOR_EXHALE,
    }
}

#[animate]
pub struct SessionAnimator {
    #[once(duration = 800, easing = cubic_in_out)]
    pub color_r: f64,

    #[once(duration = 800, easing = cubic_in_out)]
    pub color_g: f64,

    #[once(duration = 800, easing = cubic_in_out)]
    pub color_b: f64,

    #[once(duration = 500)]
    pub phase_label: String,

    #[alternate(duration = 900, easing = quad_in_out)]
    pub hold_pulse: f64,
}

impl SessionAnimator {
    pub fn for_phase(style: &PhaseStyle, label: &str) -> Self {
        let (r, g, b) = phase_color(style);
        let mut anim = SessionAnimator::new(r, g, b, label.to_string(), 1.0);
        anim.hold_pulse.set(0.65);
        anim
    }
}
