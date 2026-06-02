use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::breathing::BreathingEngine;
use super::hold::{best_hold_seconds, BreathHoldAttempt};

#[derive(Clone, Debug)]
pub struct SessionManager {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub engine: BreathingEngine,
    pub events: Vec<SessionEvent>,
    pub hold_attempts: Vec<BreathHoldAttempt>,
    final_status: Option<SessionOutcome>,
}

#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event: EventKind,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Start,
    Pause,
    Resume,
    HoldStart,
    HoldEnd,
    Complete,
    Abandon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionOutcome {
    Completed,
    Abandoned,
}

impl SessionManager {
    pub fn new(pattern: &'static super::patterns::Pattern, duration_secs: f64, tempo: f64) -> Self {
        let session_id = Uuid::new_v4();
        let start_time = Utc::now();

        let mut events = Vec::new();
        events.push(SessionEvent {
            timestamp: start_time,
            event: EventKind::Start,
            details: format!(
                "Session started: {} at tempo {}",
                pattern.display_name, tempo
            ),
        });

        Self {
            session_id,
            start_time,
            engine: BreathingEngine::new(pattern, tempo, duration_secs),
            events,
            hold_attempts: Vec::new(),
            final_status: None,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.engine.toggle_pause();
        let event_kind = if self.engine.is_paused {
            EventKind::Pause
        } else {
            EventKind::Resume
        };

        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event: event_kind,
            details: format!("Paused at {:?}", self.engine.total_elapsed_secs),
        });
    }

    pub fn start_hold(&mut self, timestamp: DateTime<Utc>) {
        self.events.push(SessionEvent {
            timestamp,
            event: EventKind::HoldStart,
            details: format!(
                "Breath hold started at {:.1}s",
                self.engine.total_elapsed_secs
            ),
        });
    }

    pub fn finish_hold(&mut self, attempt: BreathHoldAttempt) {
        self.events.push(SessionEvent {
            timestamp: attempt.ended_at,
            event: EventKind::HoldEnd,
            details: format!("Breath hold ended at {:.1}s", attempt.duration_secs),
        });
        self.hold_attempts.push(attempt);
    }

    pub fn best_hold_seconds(&self) -> Option<f64> {
        best_hold_seconds(&self.hold_attempts)
    }

    pub fn hold_attempt_count(&self) -> u32 {
        self.hold_attempts.len() as u32
    }

    pub fn session_status(&self) -> Option<SessionOutcome> {
        self.final_status
    }

    pub fn complete(&mut self) {
        if self.final_status.is_some() {
            return;
        }

        self.final_status = Some(SessionOutcome::Completed);
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event: EventKind::Complete,
            details: format!(
                "Session completed after {:?}s",
                self.engine.total_elapsed_secs
            ),
        });
    }

    pub fn abandon(&mut self) {
        if self.final_status.is_some() {
            return;
        }

        self.final_status = Some(SessionOutcome::Abandoned);
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event: EventKind::Abandon,
            details: format!(
                "Session abandoned after {:?}s",
                self.engine.total_elapsed_secs
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PATTERNS;

    #[test]
    fn tracks_multiple_hold_attempts_and_best_duration() {
        let mut manager = SessionManager::new(&PATTERNS[0], 60.0, 1.0);
        let now = Utc::now();

        manager.start_hold(now);
        manager.finish_hold(BreathHoldAttempt {
            started_at: now,
            ended_at: now,
            duration_secs: 18.2,
        });
        manager.finish_hold(BreathHoldAttempt {
            started_at: now,
            ended_at: now,
            duration_secs: 24.4,
        });

        assert_eq!(manager.hold_attempt_count(), 2);
        assert_eq!(manager.best_hold_seconds(), Some(24.4));
    }

    #[test]
    fn preserves_final_status_once_set() {
        let mut manager = SessionManager::new(&PATTERNS[0], 60.0, 1.0);
        manager.abandon();
        manager.complete();

        assert_eq!(manager.session_status(), Some(SessionOutcome::Abandoned));
    }
}
