// Breathing pattern definitions.
// Mirror lives at breath4life-web/src/patterns.ts. Sync manually on changes.
// Web side carries an extra optional `channel: "nose" | "mouth"` field per phase.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStyle {
    Rising,
    Falling,
    Steady,
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    pub name: &'static str,
    pub duration_secs: f64,
    pub style: PhaseStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct Pattern {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub phases: &'static [Phase],
}

// Pre-built patterns
pub const PATTERN_478: Pattern = Pattern {
    id: "478",
    display_name: "4-7-8 Breathing",
    description: "Inhale 4s, Hold 7s, Exhale 8s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 4.0,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Hold",
            duration_secs: 7.0,
            style: PhaseStyle::Steady,
        },
        Phase {
            name: "Exhale",
            duration_secs: 8.0,
            style: PhaseStyle::Falling,
        },
    ],
};

pub const PATTERN_BOX: Pattern = Pattern {
    id: "box",
    display_name: "Box Breathing",
    description: "Inhale 4s, Hold 4s, Exhale 4s, Hold 4s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 4.0,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Hold",
            duration_secs: 4.0,
            style: PhaseStyle::Steady,
        },
        Phase {
            name: "Exhale",
            duration_secs: 4.0,
            style: PhaseStyle::Falling,
        },
        Phase {
            name: "Hold",
            duration_secs: 4.0,
            style: PhaseStyle::Steady,
        },
    ],
};

pub const PATTERN_DIAPHRAGMATIC: Pattern = Pattern {
    id: "diaphragmatic",
    display_name: "Diaphragmatic Breathing",
    description: "Inhale 4s, Exhale 6s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 4.0,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Exhale",
            duration_secs: 6.0,
            style: PhaseStyle::Falling,
        },
    ],
};

pub const PATTERN_BREATH_OF_FIRE: Pattern = Pattern {
    id: "breath_of_fire",
    display_name: "Breath of Fire",
    description: "Rapid Kapalabhati: passive inhale 0.5s, sharp exhale 0.5s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 0.5,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Exhale",
            duration_secs: 0.5,
            style: PhaseStyle::Falling,
        },
    ],
};

pub const PATTERN_BHASTRIKA: Pattern = Pattern {
    id: "bhastrika",
    display_name: "Bhastrika (Bellows Breath)",
    description: "Forceful equal breath: Inhale 1s, Exhale 1s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 1.0,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Exhale",
            duration_secs: 1.0,
            style: PhaseStyle::Falling,
        },
    ],
};

pub const PATTERN_STIMULATING: Pattern = Pattern {
    id: "stimulating",
    display_name: "Stimulating Breath",
    description: "Rapid 3-part energizer: Inhale 0.4s, Exhale 0.4s",
    phases: &[
        Phase {
            name: "Inhale",
            duration_secs: 0.4,
            style: PhaseStyle::Rising,
        },
        Phase {
            name: "Exhale",
            duration_secs: 0.4,
            style: PhaseStyle::Falling,
        },
    ],
};

pub const PATTERNS: &[Pattern] = &[
    PATTERN_478,
    PATTERN_BOX,
    PATTERN_DIAPHRAGMATIC,
    PATTERN_BREATH_OF_FIRE,
    PATTERN_BHASTRIKA,
    PATTERN_STIMULATING,
];

/// Visual fill ratio (0.0 = empty, 1.0 = full) for a given phase + progress.
///
/// `Steady` resolves to the last extreme set by a preceding non-Steady phase:
/// full after `Rising`, empty after `Falling`. Walks backwards cyclically so
/// Hold(Out) after Exhale renders empty, fixing the "reset to inhale" jump.
pub fn fill_ratio(phases: &[Phase], idx: usize, progress: f64) -> f64 {
    match phases[idx].style {
        PhaseStyle::Rising => progress,
        PhaseStyle::Falling => 1.0 - progress,
        PhaseStyle::Steady => {
            let n = phases.len();
            for offset in 1..=n {
                let i = (idx + n - offset) % n;
                match phases[i].style {
                    PhaseStyle::Rising => return 1.0,
                    PhaseStyle::Falling => return 0.0,
                    PhaseStyle::Steady => continue,
                }
            }
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rising_returns_progress() {
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 0, 0.0), 0.0);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 0, 0.5), 0.5);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 0, 1.0), 1.0);
    }

    #[test]
    fn falling_returns_inverse_progress() {
        // Box phase 2 = Exhale (Falling)
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 2, 0.0), 1.0);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 2, 0.5), 0.5);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 2, 1.0), 0.0);
    }

    #[test]
    fn hold_in_after_inhale_is_full() {
        // Box phase 1 = Hold (In) after Rising
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 1, 0.0), 1.0);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 1, 0.7), 1.0);
    }

    #[test]
    fn hold_out_after_exhale_is_empty() {
        // Box phase 3 = Hold (Out) after Falling — must be 0.0, not max
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 3, 0.0), 0.0);
        assert_eq!(fill_ratio(PATTERN_BOX.phases, 3, 0.7), 0.0);
    }

    #[test]
    fn pattern_478_hold_after_inhale_is_full() {
        // 4-7-8 phase 1 = Hold (after Inhale Rising)
        assert_eq!(fill_ratio(PATTERN_478.phases, 1, 0.5), 1.0);
    }
}
