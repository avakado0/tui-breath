use chrono::{Duration, Utc};

use crate::app::TimeFrame;
use crate::storage::schema::IndexEntry;

pub fn hold_series(sessions: &[IndexEntry], frame: TimeFrame) -> Vec<(f64, f64)> {
    match frame {
        TimeFrame::SevenDays | TimeFrame::ThirtyDays => {
            let days = match frame {
                TimeFrame::SevenDays => 7,
                TimeFrame::ThirtyDays => 30,
                TimeFrame::All => unreachable!(),
            };
            let cutoff = Utc::now() - Duration::days(days);
            sessions
                .iter()
                .filter(|s| s.start_time >= cutoff && s.best_breath_hold_seconds.is_some())
                .enumerate()
                .map(|(idx, s)| (idx as f64, s.best_breath_hold_seconds.unwrap()))
                .collect()
        }
        TimeFrame::All => sessions
            .iter()
            .filter(|s| s.best_breath_hold_seconds.is_some())
            .enumerate()
            .map(|(idx, s)| (idx as f64, s.best_breath_hold_seconds.unwrap()))
            .collect(),
    }
}

pub fn sessions_per_day(sessions: &[IndexEntry], frame: TimeFrame) -> Vec<(f64, f64)> {
    use std::collections::BTreeMap;

    let (cutoff, _all_time) = match frame {
        TimeFrame::SevenDays => (Utc::now() - Duration::days(7), false),
        TimeFrame::ThirtyDays => (Utc::now() - Duration::days(30), false),
        TimeFrame::All => (Utc::now() - Duration::days(36500), true),
    };

    let filtered: Vec<_> = sessions
        .iter()
        .filter(|s| s.start_time >= cutoff)
        .collect();

    let mut day_buckets: BTreeMap<String, f64> = BTreeMap::new();
    for session in filtered {
        let day_key = session.start_time.format("%Y-%m-%d").to_string();
        *day_buckets.entry(day_key).or_insert(0.0) += 1.0;
    }

    day_buckets
        .into_iter()
        .enumerate()
        .map(|(idx, (_day, count))| (idx as f64, count))
        .collect()
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
        let series = hold_series(&[], TimeFrame::All);
        assert!(series.is_empty());
    }

    #[test]
    fn hold_series_filters_none() {
        let sessions = vec![test_entry("1", None), test_entry("2", None)];
        let series = hold_series(&sessions, TimeFrame::All);
        assert!(series.is_empty());
    }

    #[test]
    fn hold_series_all_includes_all_with_hold() {
        let sessions = vec![
            test_entry("1", Some(10.5)),
            test_entry("2", Some(15.3)),
            test_entry("3", Some(20.0)),
        ];
        let series = hold_series(&sessions, TimeFrame::All);
        assert_eq!(series.len(), 3);
        assert_eq!(series[0], (0.0, 10.5));
        assert_eq!(series[1], (1.0, 15.3));
        assert_eq!(series[2], (2.0, 20.0));
    }

    #[test]
    fn sessions_per_day_empty() {
        let series = sessions_per_day(&[], TimeFrame::All);
        assert!(series.is_empty());
    }

    #[test]
    fn sessions_per_day_single_day() {
        let sessions = vec![
            test_entry("1", Some(10.0)),
            test_entry("2", Some(15.0)),
            test_entry("3", Some(20.0)),
        ];
        let series = sessions_per_day(&sessions, TimeFrame::All);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].1, 3.0);
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

        let series = sessions_per_day(&entries, TimeFrame::All);
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].1, 1.0);
        assert_eq!(series[1].1, 1.0);
        assert_eq!(series[2].1, 2.0);
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

        let series = sessions_per_day(&sessions, TimeFrame::SevenDays);
        assert_eq!(series.len(), 2);
    }
}
