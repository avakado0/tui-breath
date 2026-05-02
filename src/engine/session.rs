use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::breathing::BreathingEngine;

#[derive(Clone, Debug)]
pub struct SessionManager {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub engine: BreathingEngine,
    pub events: Vec<SessionEvent>,
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
    Complete,
    Abandon,
}

impl SessionManager {
    pub fn new(pattern: &'static super::patterns::Pattern, duration_secs: f64, tempo: f64) -> Self {
        let session_id = Uuid::new_v4();
        let start_time = Utc::now();

        let mut events = Vec::new();
        events.push(SessionEvent {
            timestamp: start_time,
            event: EventKind::Start,
            details: format!("Session started: {} at tempo {}", pattern.display_name, tempo),
        });

        Self {
            session_id,
            start_time,
            engine: BreathingEngine::new(pattern, tempo, duration_secs),
            events,
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

    pub fn complete(&mut self) {
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event: EventKind::Complete,
            details: format!("Session completed after {:?}s", self.engine.total_elapsed_secs),
        });
    }

    pub fn abandon(&mut self) {
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event: EventKind::Abandon,
            details: format!("Session abandoned after {:?}s", self.engine.total_elapsed_secs),
        });
    }
}
