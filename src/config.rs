use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub org_id: String,
    pub session_key: String,
    pub cf_clearance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cf_bm: Option<String>,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config/claude-usage-monitor/config.toml"))
    }

    pub fn from_env() -> Option<Self> {
        fn nonempty(key: &str) -> Option<String> {
            std::env::var(key).ok().filter(|v| !v.is_empty())
        }
        let org_id = nonempty("CLAUDE_ORG_ID")?;
        let session_key = nonempty("CLAUDE_SESSION_KEY")?;
        let cf_clearance = nonempty("CLAUDE_CF_CLEARANCE")?;
        let cf_bm = nonempty("CLAUDE_CF_BM");
        Some(Config { org_id, session_key, cf_clearance, cf_bm })
    }

    pub fn from_file() -> Option<Self> {
        let path = Self::path()?;
        let text = fs::read_to_string(&path).ok()?;
        toml::from_str(&text).ok()
    }

    pub fn load() -> Option<Self> {
        Self::from_env().or_else(Self::from_file)
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::path().context("HOME is not set")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).ok();
        }
        let text = toml::to_string_pretty(self).context("serialize config")?;
        fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 600 {}", path.display()))?;
        Ok(path)
    }
}
