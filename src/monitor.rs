use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::{json, Value};

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{AggregatedUsage, CodexStatus, ScopedWindow, UsageResponse, WindowUsage};

pub const REFRESH: Duration = Duration::from_secs(2);

pub enum Update {
    Claude(Result<Value>),
    Codex(Result<WindowUsage>),
}

#[derive(Default)]
pub struct Snapshot {
    pub claude: Option<Value>,
    pub claude_error: Option<String>,
    pub claude_updated_at: Option<String>,
    pub codex: CodexStatus,
}

impl Snapshot {
    pub fn apply(&mut self, update: Update) {
        let now = chrono::Utc::now().to_rfc3339();
        match update {
            Update::Claude(Ok(data)) => {
                self.claude = Some(data);
                self.claude_error = None;
                self.claude_updated_at = Some(now);
            }
            Update::Claude(Err(e)) => self.claude_error = Some(e.to_string()),
            Update::Codex(Ok(weekly)) => {
                self.codex.weekly = Some(weekly);
                self.codex.error = None;
                self.codex.updated_at = Some(now);
            }
            Update::Codex(Err(e)) => self.codex.error = Some(e.to_string()),
        }
    }

    pub fn json(&self) -> Value {
        let mut value = self.claude.clone().unwrap_or_else(|| json!({}));
        value["claude_error"] = json!(self.claude_error);
        value["claude_updated_at"] = json!(self.claude_updated_at);
        value["codex"] = json!(self.codex);
        value
    }

    pub fn aggregated(&self) -> AggregatedUsage {
        let mut usage = AggregatedUsage {
            codex: self.codex.clone(),
            ..Default::default()
        };
        if let Some(data) = self
            .claude
            .clone()
            .and_then(|v| serde_json::from_value::<UsageResponse>(v).ok())
        {
            usage.claude_available = true;
            usage.five_hour_util = data.five_hour.utilization;
            usage.five_hour_resets_at = data.five_hour.resets_at;
            usage.seven_day_util = data.seven_day.utilization;
            usage.seven_day_resets_at = data.seven_day.resets_at;
            usage.scoped_weekly = data
                .limits
                .into_iter()
                .filter(|l| l.kind == "weekly_scoped")
                .map(|l| ScopedWindow {
                    label: l
                        .scope
                        .and_then(|s| s.model)
                        .and_then(|m| m.display_name)
                        .unwrap_or_else(|| "Scoped".into()),
                    util: l.percent.unwrap_or(0.0),
                    resets_at: l.resets_at,
                })
                .collect();
        }
        usage
    }
}

// Independent workers keep network timeouts out of the terminal event loop.
// Dropping the monitor stops future polls (an in-flight request may finish).
pub struct Monitor {
    pub updates: Receiver<Update>,
    _stop: Vec<Sender<()>>,
}

impl Monitor {
    pub fn start(cfg: Config, repeat: bool) -> Self {
        let (tx, updates) = mpsc::channel();
        let mut stop = Vec::new();
        for codex in [false, true] {
            let cfg = cfg.clone();
            let tx = tx.clone();
            let (stop_tx, stop_rx) = mpsc::channel();
            stop.push(stop_tx);
            std::thread::spawn(move || {
                let api = match ApiClient::new(cfg) {
                    Ok(api) => api,
                    Err(_) => {
                        let error = anyhow::anyhow!("could not initialize HTTP client");
                        let _ = tx.send(if codex {
                            Update::Codex(Err(error))
                        } else {
                            Update::Claude(Err(error))
                        });
                        return;
                    }
                };
                loop {
                    let started = Instant::now();
                    let update = if codex {
                        Update::Codex(api.fetch_codex())
                    } else {
                        Update::Claude(api.fetch_claude())
                    };
                    if tx.send(update).is_err() || !repeat {
                        break;
                    }
                    if stop_rx.recv_timeout(REFRESH.saturating_sub(started.elapsed()))
                        != Err(mpsc::RecvTimeoutError::Timeout)
                    {
                        break;
                    }
                }
            });
        }
        Self {
            updates,
            _stop: stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_failures_preserve_other_provider_and_mark_stale() {
        let mut snapshot = Snapshot::default();
        snapshot.apply(Update::Claude(Ok(
            json!({"five_hour": {"utilization": 12}, "seven_day": {"utilization": 34}}),
        )));
        snapshot.apply(Update::Codex(Ok(WindowUsage {
            utilization: 13.0,
            resets_at: None,
        })));
        let updated = snapshot.codex.updated_at.clone();
        snapshot.apply(Update::Codex(Err(anyhow::anyhow!("auth 401"))));
        assert_eq!(snapshot.codex.weekly.as_ref().unwrap().utilization, 13.0);
        assert_eq!(snapshot.codex.updated_at, updated);
        assert_eq!(snapshot.aggregated().five_hour_util, 12.0);
        snapshot.apply(Update::Claude(Err(anyhow::anyhow!("http 403"))));
        snapshot.apply(Update::Codex(Ok(WindowUsage {
            utilization: 14.0,
            resets_at: None,
        })));
        assert!(snapshot.codex.error.is_none());
        assert!(snapshot.claude_error.is_some());
        assert_eq!(snapshot.json()["codex"]["weekly"]["utilization"], 14.0);
    }

    #[test]
    fn first_fetch_failure_never_reports_zero_usage() {
        let mut snapshot = Snapshot::default();
        snapshot.apply(Update::Claude(Err(anyhow::anyhow!("offline"))));
        snapshot.apply(Update::Codex(Err(anyhow::anyhow!("not configured"))));
        assert!(!snapshot.aggregated().claude_available);
        assert!(snapshot.json()["codex"]["weekly"].is_null());
        assert!(snapshot.json().get("five_hour").is_none());
    }
}
