use chrono::{Duration, Utc};

use crate::app::TimeFrame;
use crate::storage::schema::IndexEntry;

pub struct ChartData {
    pub points: Vec<(f64, f64)>,
    pub x_labels: Vec<(f64, String)>,
}

pub fn hold_series(sessions: &[IndexEntry], frame: TimeFrame) -> ChartData {
    use std::collections::BTreeMap;

    let days = match frame {
        TimeFrame::SevenDays => 7,
        TimeFrame::ThirtyDays => 30,
        TimeFrame::All => return all_time_hold_series(sessions),
    };

    let cutoff = Utc::now() - Duration::days(days);
    let mut day_buckets: BTreeMap<String, f64> = BTreeMap::new();

    for session in sessions {
        if session.start_time >= cutoff && session.best_breath_hold_seconds.is_some() {
            let day_key = session.start_time.format("%Y-%m-%d").to_string();
            day_buckets
                .entry(day_key)
                .or_insert_with(|| session.best_breath_hold_seconds.unwrap());
        }
    }

    let points: Vec<(f64, f64)> = day_buckets
        .iter()
        .enumerate()
        .map(|(idx, (_day, val))| (idx as f64, *val))
        .collect();

    let x_labels = generate_x_labels(day_buckets.keys().cloned().collect(), &points);
    ChartData { points, x_labels }
}

fn all_time_hold_series(sessions: &[IndexEntry]) -> ChartData {
    let points: Vec<(f64, f64)> = sessions
        .iter()
        .filter(|s| s.best_breath_hold_seconds.is_some())
        .enumerate()
        .map(|(idx, s)| (idx as f64, s.best_breath_hold_seconds.unwrap()))
        .collect();

    let x_labels = if points.len() > 10 {
        let step = (points.len() / 4).max(1);
        (0..points.len())
            .step_by(step)
            .map(|i| (i as f64, format!("#{}", i + 1)))
            .collect()
    } else {
        (0..points.len())
            .map(|i| (i as f64, format!("#{}", i + 1)))
            .collect()
    };

    ChartData { points, x_labels }
}

pub fn sessions_per_day(sessions: &[IndexEntry], frame: TimeFrame) -> ChartData {
    use std::collections::BTreeMap;

    let days = match frame {
        TimeFrame::SevenDays => 7,
        TimeFrame::ThirtyDays => 30,
        TimeFrame::All => return all_time_sessions_per_day(sessions),
    };

    let cutoff = Utc::now() - Duration::days(days);
    let mut day_buckets: BTreeMap<String, f64> = BTreeMap::new();

    for session in sessions {
        if session.start_time >= cutoff {
            let day_key = session.start_time.format("%Y-%m-%d").to_string();
            *day_buckets.entry(day_key).or_insert(0.0) += 1.0;
        }
    }

    let points: Vec<(f64, f64)> = day_buckets
        .iter()
        .enumerate()
        .map(|(idx, (_day, count))| (idx as f64, *count))
        .collect();

    let days_vec: Vec<String> = day_buckets.keys().cloned().collect();
    let x_labels = generate_x_labels(days_vec, &points);
    ChartData { points, x_labels }
}

fn all_time_sessions_per_day(sessions: &[IndexEntry]) -> ChartData {
    use std::collections::BTreeMap;

    let mut day_buckets: BTreeMap<String, f64> = BTreeMap::new();
    for session in sessions {
        let day_key = session.start_time.format("%Y-%m-%d").to_string();
        *day_buckets.entry(day_key).or_insert(0.0) += 1.0;
    }

    let points: Vec<(f64, f64)> = day_buckets
        .iter()
        .enumerate()
        .map(|(idx, (_day, count))| (idx as f64, *count))
        .collect();

    let days_vec: Vec<String> = day_buckets.keys().cloned().collect();
    let x_labels = generate_x_labels(days_vec, &points);
    ChartData { points, x_labels }
}

fn generate_x_labels(days: Vec<String>, _points: &[(f64, f64)]) -> Vec<(f64, String)> {
    if days.is_empty() {
        return vec![];
    }

    if days.len() <= 5 {
        days.iter()
            .enumerate()
            .map(|(idx, day)| (idx as f64, day.clone()))
            .collect()
    } else {
        let step = (days.len() / 4).max(1);
        (0..days.len())
            .step_by(step)
            .map(|i| (i as f64, days[i].clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_entry(id: &str, hold_secs: Option<f64>) -> IndexEntry {
        IndexEntry {
            session_id: id.to_string(),
            start_time: Utc::now(),
            status: "completed".to_string(),
            pattern_id: "test".to_string(),
            duration_target: 300,
            cycles_completed: 1,
            completion_pct: 100.0,
            best_breath_hold_seconds: hold_secs,
            breath_hold_attempt_count: if hold_secs.is_some() { 1 } else { 0 },
        }
    }

    #[test]
    fn hold_series_empty() {
        let data = hold_series(&[], TimeFrame::All);
        assert!(data.points.is_empty());
    }

    #[test]
    fn hold_series_filters_none() {
        let sessions = vec![test_entry("1", None), test_entry("2", None)];
        let data = hold_series(&sessions, TimeFrame::All);
        assert!(data.points.is_empty());
    }

    #[test]
    fn hold_series_all_includes_all_with_hold() {
        let sessions = vec![
            test_entry("1", Some(10.5)),
            test_entry("2", Some(15.3)),
            test_entry("3", Some(20.0)),
        ];
        let data = hold_series(&sessions, TimeFrame::All);
        assert_eq!(data.points.len(), 3);
        assert_eq!(data.points[0], (0.0, 10.5));
        assert_eq!(data.points[1], (1.0, 15.3));
        assert_eq!(data.points[2], (2.0, 20.0));
    }

    #[test]
    fn sessions_per_day_empty() {
        let data = sessions_per_day(&[], TimeFrame::All);
        assert!(data.points.is_empty());
    }

    #[test]
    fn sessions_per_day_single_day() {
        let sessions = vec![
            test_entry("1", Some(10.0)),
            test_entry("2", Some(15.0)),
            test_entry("3", Some(20.0)),
        ];
        let data = sessions_per_day(&sessions, TimeFrame::All);
        assert_eq!(data.points.len(), 1);
        assert_eq!(data.points[0].1, 3.0);
    }

    #[test]
    fn sessions_per_day_multiple_days() {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let two_days_ago = now - Duration::days(2);

        let entries = vec![
            IndexEntry {
                start_time: now,
                ..test_entry("1", Some(10.0))
            },
            IndexEntry {
                start_time: now,
                ..test_entry("2", Some(15.0))
            },
            IndexEntry {
                start_time: yesterday,
                ..test_entry("3", Some(20.0))
            },
            IndexEntry {
                start_time: two_days_ago,
                ..test_entry("4", Some(12.0))
            },
        ];

        let data = sessions_per_day(&entries, TimeFrame::All);
        assert_eq!(data.points.len(), 3);
        assert_eq!(data.points[0].1, 1.0);
        assert_eq!(data.points[1].1, 1.0);
        assert_eq!(data.points[2].1, 2.0);
    }

    #[test]
    fn sessions_per_day_seven_days_filters_old() {
        let now = Utc::now();
        let two_days_ago = now - Duration::days(2);
        let ten_days_ago = now - Duration::days(10);

        let sessions = vec![
            IndexEntry {
                start_time: now,
                ..test_entry("1", None)
            },
            IndexEntry {
                start_time: two_days_ago,
                ..test_entry("2", None)
            },
            IndexEntry {
                start_time: ten_days_ago,
                ..test_entry("3", None)
            },
        ];

        let data = sessions_per_day(&sessions, TimeFrame::SevenDays);
        assert_eq!(data.points.len(), 2);
    }
}
