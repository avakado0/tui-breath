// Breathing pattern definitions

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
            name: "Hold (In)",
            duration_secs: 4.0,
            style: PhaseStyle::Steady,
        },
        Phase {
            name: "Exhale",
            duration_secs: 4.0,
            style: PhaseStyle::Falling,
        },
        Phase {
            name: "Hold (Out)",
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

pub const PATTERNS: &[Pattern] = &[PATTERN_478, PATTERN_BOX, PATTERN_DIAPHRAGMATIC];
