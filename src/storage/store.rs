use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::schema::{
    BreathHoldAttemptRecord, BreathHoldData, IndexEntry, SessionRecord, SessionStatus,
};
use crate::engine::hold::BreathHoldAttempt;
use crate::engine::session::SessionOutcome;
use crate::engine::SessionManager;

pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tui_breath");
        Self::new_in(data_dir)
    }

    pub(crate) fn new_in(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("sessions"))?;

        Ok(Self { data_dir })
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.data_dir.join("sessions")
    }

    pub fn index_path(&self) -> PathBuf {
        self.data_dir.join("index.json")
    }

    pub fn save_session(&self, manager: &SessionManager) -> Result<()> {
        let sessions_dir = self.sessions_dir();
        fs::create_dir_all(&sessions_dir)?;

        let record = session_manager_to_record(manager);
        let json = serde_json::to_string_pretty(&record)?;
        let path = sessions_dir.join(format!("{}.json", record.session_id));
        fs::write(path, json)?;

        self.update_index(&record)?;

        Ok(())
    }

    pub fn load_index(&self) -> Result<Vec<IndexEntry>> {
        let index_path = self.index_path();
        if !index_path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(index_path)?;
        let entries: Vec<IndexEntry> = serde_json::from_str(&json)?;
        Ok(entries)
    }

    fn update_index(&self, record: &SessionRecord) -> Result<()> {
        let mut entries = self.load_index()?;
        entries.retain(|entry| entry.session_id != record.session_id);

        let entry = IndexEntry {
            session_id: record.session_id.clone(),
            start_time: record.start_time,
            status: format!("{:?}", record.status).to_lowercase(),
            pattern_id: record.parameters.settings.pattern_id.clone(),
            duration_target: record.parameters.duration_target,
            cycles_completed: record.parameters.settings.iterations,
            completion_pct: calculate_completion_pct(record),
            best_breath_hold_seconds: record
                .breath_hold
                .as_ref()
                .and_then(|hold| hold.best_seconds),
            breath_hold_attempt_count: record
                .breath_hold
                .as_ref()
                .map(|hold| hold.attempt_count)
                .unwrap_or(0),
        };
        entries.push(entry);

        entries.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        let json = serde_json::to_string_pretty(&entries)?;
        let path = self.index_path();
        fs::write(path, json)?;

        Ok(())
    }
}

fn session_manager_to_record(manager: &SessionManager) -> SessionRecord {
    let engine = &manager.engine;
    let pattern = engine.pattern;
    let target_secs = engine.duration_target_secs as u32;

    let breaths_per_minute = if engine.total_elapsed_secs > 0.0 {
        engine.cycle_count as f64 / engine.total_elapsed_secs * 60.0
    } else {
        0.0
    };

    let phases = engine.pattern.phases;
    let first_inhale = phases
        .iter()
        .find(|phase| phase.name.starts_with("Inhale"))
        .map(|phase| phase.duration_secs / engine.tempo)
        .unwrap_or(0.0);

    let first_exhale = phases
        .iter()
        .find(|phase| phase.name.starts_with("Exhale"))
        .map(|phase| phase.duration_secs / engine.tempo)
        .unwrap_or(0.0);

    let hold_in = phases
        .iter()
        .find(|phase| {
            phase.name.contains("Hold (In)")
                || phase.name.contains("Hold In")
                || phase.name == "Hold"
        })
        .map(|phase| phase.duration_secs / engine.tempo);

    let hold_out = phases
        .iter()
        .rev()
        .find(|phase| phase.name.contains("Hold (Out)") || phase.name.contains("Hold Out"))
        .map(|phase| phase.duration_secs / engine.tempo);

    SessionRecord {
        session_id: manager.session_id.to_string(),
        start_time: manager.start_time,
        end_time: Some(chrono::Utc::now()),
        status: match manager
            .session_status()
            .unwrap_or(SessionOutcome::Completed)
        {
            SessionOutcome::Completed => SessionStatus::Completed,
            SessionOutcome::Abandoned => SessionStatus::Abandoned,
        },
        session_type: "breathing".to_string(),
        parameters: crate::storage::schema::SessionParameters {
            duration_target: target_secs,
            actual_duration_secs: engine.total_elapsed_secs,
            settings: crate::storage::schema::SessionSettings {
                rate: breaths_per_minute,
                phase_parameters: crate::storage::schema::PhaseParameters {
                    inhalation_time: first_inhale,
                    exhalation_time: first_exhale,
                    hold_in_time: hold_in,
                    hold_out_time: hold_out,
                },
                iterations: engine.cycle_count,
                pattern_id: pattern.id.to_string(),
                tempo: engine.tempo,
            },
        },
        breath_hold: breath_hold_data(&manager.hold_attempts),
        history: manager
            .events
            .iter()
            .map(|event| crate::storage::schema::HistoryEvent {
                timestamp: event.timestamp,
                event: format!("{:?}", event.event),
                details: serde_json::json!({"details": event.details}),
            })
            .collect(),
    }
}

fn breath_hold_data(attempts: &[BreathHoldAttempt]) -> Option<BreathHoldData> {
    if attempts.is_empty() {
        return None;
    }

    Some(BreathHoldData {
        best_seconds: attempts
            .iter()
            .map(|attempt| attempt.duration_secs)
            .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)),
        attempt_count: attempts.len() as u32,
        attempts: attempts
            .iter()
            .map(|attempt| BreathHoldAttemptRecord {
                started_at: attempt.started_at,
                ended_at: attempt.ended_at,
                duration_secs: attempt.duration_secs,
            })
            .collect(),
    })
}

fn calculate_completion_pct(record: &SessionRecord) -> f64 {
    let target = record.parameters.duration_target as f64;
    if target > 0.0 {
        (record.parameters.actual_duration_secs / target * 100.0).min(100.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::hold::BreathHoldAttempt;
    use crate::engine::PATTERNS;
    use chrono::Utc;

    #[test]
    fn record_uses_best_hold_and_active_duration_completion() {
        let mut manager = SessionManager::new(&PATTERNS[0], 120.0, 1.0);
        manager.engine.total_elapsed_secs = 60.0;
        manager.hold_attempts.push(BreathHoldAttempt {
            started_at: Utc::now(),
            ended_at: Utc::now(),
            duration_secs: 18.5,
        });
        manager.hold_attempts.push(BreathHoldAttempt {
            started_at: Utc::now(),
            ended_at: Utc::now(),
            duration_secs: 23.1,
        });
        manager.complete();

        let record = session_manager_to_record(&manager);

        assert_eq!(record.status, SessionStatus::Completed);
        assert_eq!(record.breath_hold.as_ref().unwrap().attempt_count, 2);
        assert_eq!(
            record.breath_hold.as_ref().unwrap().best_seconds,
            Some(23.1)
        );
        assert!((calculate_completion_pct(&record) - 50.0).abs() < 0.001);
    }

    #[test]
    fn abandoned_session_status_is_preserved() {
        let mut manager = SessionManager::new(&PATTERNS[0], 60.0, 1.0);
        manager.abandon();

        let record = session_manager_to_record(&manager);
        assert_eq!(record.status, SessionStatus::Abandoned);
    }

    #[test]
    fn old_index_entries_without_hold_fields_still_deserialize() {
        let json = r#"{
            "session_id": "abc",
            "start_time": "2026-06-02T12:00:00Z",
            "status": "completed",
            "pattern_id": "box",
            "duration_target": 60,
            "cycles_completed": 4,
            "completion_pct": 100.0
        }"#;

        let entry: IndexEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.best_breath_hold_seconds, None);
        assert_eq!(entry.breath_hold_attempt_count, 0);
    }
}
