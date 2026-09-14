use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub org_id: String,
    pub session_key: String,
    pub cf_clearance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cf_bm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_cookie: Option<String>,
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
        Some(Config {
            org_id,
            session_key,
            cf_clearance,
            cf_bm,
            ..Default::default()
        })
    }

    pub fn from_file() -> Option<Self> {
        let path = Self::path()?;
        let text = fs::read_to_string(&path).ok()?;
        toml::from_str(&text).ok()
    }

    pub fn load() -> Option<Self> {
        let mut cfg = Self::from_file().or_else(Self::from_env)?;
        if let Some(env) = Self::from_env() {
            cfg.org_id = env.org_id;
            cfg.session_key = env.session_key;
            cfg.cf_clearance = env.cf_clearance;
            cfg.cf_bm = env.cf_bm;
        }
        for (key, field) in [
            ("CODEX_ACCESS_TOKEN", &mut cfg.codex_access_token),
            ("CODEX_ACCOUNT_ID", &mut cfg.codex_account_id),
            ("CODEX_COOKIE", &mut cfg.codex_cookie),
        ] {
            if let Ok(value) = std::env::var(key) {
                *field = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_owned())
                };
            }
        }
        Some(cfg)
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::path().context("HOME is not set")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).ok();
        }
        let text = toml::to_string_pretty(self).context("serialize config")?;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
        file.write_all(text.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 600 {}", path.display()))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_claude_config_still_loads() {
        let cfg: Config =
            toml::from_str("org_id='org'\nsession_key='session'\ncf_clearance='clearance'\n")
                .unwrap();
        assert!(cfg.codex_access_token.is_none());
        assert!(cfg.codex_account_id.is_none());
        assert!(cfg.codex_cookie.is_none());
    }
}
