use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    #[serde(rename = "type")]
    pub session_type: String,
    pub parameters: SessionParameters,
    #[serde(default)]
    pub breath_hold: Option<BreathHoldData>,
    pub history: Vec<HistoryEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParameters {
    pub duration_target: u32,
    #[serde(default)]
    pub actual_duration_secs: f64,
    pub settings: SessionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub rate: f64,
    pub phase_parameters: PhaseParameters,
    pub iterations: u32,
    pub pattern_id: String,
    pub tempo: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseParameters {
    pub inhalation_time: f64,
    pub exhalation_time: f64,
    pub hold_in_time: Option<f64>,
    pub hold_out_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathHoldData {
    #[serde(default)]
    pub best_seconds: Option<f64>,
    #[serde(default)]
    pub attempt_count: u32,
    #[serde(default)]
    pub attempts: Vec<BreathHoldAttemptRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathHoldAttemptRecord {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub status: String,
    pub pattern_id: String,
    pub duration_target: u32,
    pub cycles_completed: u32,
    pub completion_pct: f64,
    #[serde(default)]
    pub best_breath_hold_seconds: Option<f64>,
    #[serde(default)]
    pub breath_hold_attempt_count: u32,
}
