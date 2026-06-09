use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct Cache {
    checked_at: DateTime<Utc>,
    latest_version: String,
}

fn cache_path() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tui_breath")
        .join("last_check.json")
}

fn cached_version(path: &Path) -> Option<String> {
    let data = std::fs::read_to_string(path).ok()?;
    let cache: Cache = serde_json::from_str(&data).ok()?;
    if Utc::now() - cache.checked_at < Duration::hours(24) {
        Some(cache.latest_version)
    } else {
        None
    }
}

fn fetch_latest() -> Result<String> {
    let body: serde_json::Value = ureq::get("https://crates.io/api/v1/crates/tui_breath")
        .set(
            "User-Agent",
            &format!(
                "tui_breath/{} (update-check)",
                env!("CARGO_PKG_VERSION")
            ),
        )
        .call()?
        .into_json()?;
    Ok(body["crate"]["newest_version"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing field"))?
        .to_string())
}

fn write_cache(path: &Path, version: &str) {
    if let Ok(data) = serde_json::to_string_pretty(&Cache {
        checked_at: Utc::now(),
        latest_version: version.to_string(),
    }) {
        let _ = std::fs::write(path, data);
    }
}

fn is_newer(v: &str) -> bool {
    fn parse(s: &str) -> (u32, u32, u32) {
        let mut parts = s.splitn(3, '.').map(|p| p.parse().unwrap_or(0));
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    }
    parse(v) > parse(env!("CARGO_PKG_VERSION"))
}

pub async fn check_for_update() -> Option<String> {
    let path = cache_path();
    let version = if let Some(cached) = cached_version(&path) {
        cached
    } else {
        let p = path.clone();
        tokio::task::spawn_blocking(move || {
            fetch_latest().ok().map(|v| {
                write_cache(&p, &v);
                v
            })
        })
        .await
        .ok()
        .flatten()?
    };
    if is_newer(&version) {
        Some(version)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.3.3")); // current is 0.3.2
        assert!(is_newer("0.4.0"));
        assert!(is_newer("1.0.0"));
        assert!(!is_newer("0.3.2")); // same version
        assert!(!is_newer("0.3.1"));
        assert!(!is_newer("0.2.0"));
    }

    #[test]
    fn test_is_newer_malformed() {
        assert!(!is_newer("invalid"));
        assert!(!is_newer(""));
    }
}
