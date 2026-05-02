use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::engine::SessionManager;
use super::schema::{SessionRecord, SessionStatus, IndexEntry};

pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tui_breath");

        // Ensure directories exist
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("sessions"))?;

        Ok(Self { data_dir })
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
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

        // Update index
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

    pub fn load_session(&self, session_id: &str) -> Result<SessionRecord> {
        let path = self.sessions_dir().join(format!("{}.json", session_id));
        let json = fs::read_to_string(path)?;
        let record: SessionRecord = serde_json::from_str(&json)?;
        Ok(record)
    }

    fn update_index(&self, record: &SessionRecord) -> Result<()> {
        let mut entries = self.load_index()?;

        // Remove existing entry with same ID
        entries.retain(|e| e.session_id != record.session_id);

        // Add new entry
        let entry = IndexEntry {
            session_id: record.session_id.clone(),
            start_time: record.start_time,
            status: format!("{:?}", record.status).to_lowercase(),
            pattern_id: record.parameters.settings.pattern_id.clone(),
            duration_target: record.parameters.duration_target,
            cycles_completed: record.parameters.settings.iterations,
            completion_pct: calculate_completion_pct(&record),
        };
        entries.push(entry);

        // Sort by start_time descending (newest first)
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

    let elapsed_secs = engine.total_elapsed_secs as u32;
    let target_secs = engine.duration_target_secs as u32;

    let breaths_per_minute = if engine.total_elapsed_secs > 0.0 {
        (engine.cycle_count as f64 / engine.total_elapsed_secs * 60.0)
    } else {
        0.0
    };

    // Extract phase durations
    let phases = engine.pattern.phases;
    let first_inhale = phases
        .iter()
        .find(|p| p.name.starts_with("Inhale"))
        .map(|p| p.duration_secs / engine.tempo)
        .unwrap_or(0.0);

    let first_exhale = phases
        .iter()
        .find(|p| p.name.starts_with("Exhale"))
        .map(|p| p.duration_secs / engine.tempo)
        .unwrap_or(0.0);

    let hold_in = phases
        .iter()
        .find(|p| p.name.contains("Hold (In)") || p.name.contains("Hold In"))
        .map(|p| p.duration_secs / engine.tempo);

    let hold_out = phases
        .iter()
        .find(|p| p.name.contains("Hold (Out)") || p.name.contains("Hold Out"))
        .map(|p| p.duration_secs / engine.tempo);

    SessionRecord {
        session_id: manager.session_id.to_string(),
        start_time: manager.start_time,
        end_time: Some(chrono::Utc::now()),
        status: SessionStatus::Completed,
        session_type: "breathing".to_string(),
        parameters: crate::storage::schema::SessionParameters {
            duration_target: target_secs,
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
        history: manager
            .events
            .iter()
            .map(|evt| crate::storage::schema::HistoryEvent {
                timestamp: evt.timestamp,
                event: format!("{:?}", evt.event),
                details: serde_json::json!({"details": evt.details}),
            })
            .collect(),
    }
}

fn calculate_completion_pct(record: &SessionRecord) -> f64 {
    let elapsed = record
        .end_time
        .and_then(|end| {
            Some((end.timestamp() - record.start_time.timestamp()) as f64)
        })
        .unwrap_or(0.0);

    let target = record.parameters.duration_target as f64;
    if target > 0.0 {
        (elapsed / target * 100.0).min(100.0)
    } else {
        0.0
    }
}
