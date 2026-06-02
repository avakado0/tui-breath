use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct BreathHoldAttempt {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_secs: f64,
}

#[derive(Debug, Clone)]
pub struct BreathHoldRuntime {
    pub started_at: DateTime<Utc>,
    pub elapsed_secs: f64,
}

impl BreathHoldRuntime {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            started_at,
            elapsed_secs: 0.0,
        }
    }

    pub fn tick(&mut self, delta_secs: f64) {
        self.elapsed_secs += delta_secs;
    }

    pub fn finish(self, ended_at: DateTime<Utc>) -> BreathHoldAttempt {
        BreathHoldAttempt {
            started_at: self.started_at,
            ended_at,
            duration_secs: self.elapsed_secs,
        }
    }
}

pub fn best_hold_seconds(attempts: &[BreathHoldAttempt]) -> Option<f64> {
    attempts
        .iter()
        .map(|attempt| attempt.duration_secs)
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_runtime_accumulates_elapsed_time() {
        let mut runtime = BreathHoldRuntime::new(Utc::now());
        runtime.tick(1.5);
        runtime.tick(2.0);

        assert!((runtime.elapsed_secs - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn best_hold_uses_longest_attempt() {
        let now = Utc::now();
        let attempts = vec![
            BreathHoldAttempt {
                started_at: now,
                ended_at: now,
                duration_secs: 11.2,
            },
            BreathHoldAttempt {
                started_at: now,
                ended_at: now,
                duration_secs: 17.8,
            },
        ];

        assert_eq!(best_hold_seconds(&attempts), Some(17.8));
    }
}
