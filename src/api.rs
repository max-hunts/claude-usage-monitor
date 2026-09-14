use anyhow::{anyhow, Result};
use rquest::{Client, Response};
use rquest_util::Emulation;
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::models::{CodexResponse, WindowUsage};

const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

pub struct ApiClient {
    client: Client,
    runtime: Runtime,
    cfg: Config,
    codex_retry_until: std::cell::Cell<i64>,
    backoff_path: Option<std::path::PathBuf>,
}

impl ApiClient {
    pub fn new(cfg: Config) -> Result<Self> {
        let client = Client::builder()
            .emulation(Emulation::Chrome136)
            // Never forward pasted credentials to a redirect destination.
            .redirect(rquest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(8))
            .build()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            client,
            runtime,
            cfg,
            codex_retry_until: std::cell::Cell::new(0),
            backoff_path: Config::path().map(|p| p.with_file_name("codex-backoff.json")),
        })
    }

    pub fn fetch_claude(&self) -> Result<Value> {
        self.runtime.block_on(async {
            let mut cookie = format!(
                "sessionKey={}; cf_clearance={}",
                self.cfg.session_key, self.cfg.cf_clearance
            );
            if let Some(bm) = &self.cfg.cf_bm {
                cookie.push_str("; __cf_bm=");
                cookie.push_str(bm);
            }
            let response = self
                .client
                .get(format!(
                    "https://claude.ai/api/organizations/{}/usage",
                    self.cfg.org_id
                ))
                .header("accept", "application/json")
                .header("referer", "https://claude.ai/settings/usage")
                .header("cookie", cookie)
                .send()
                .await
                .map_err(|_| anyhow!("request failed or timed out"))?;
            check_status(&response)?;
            let mut data: Value = response
                .json()
                .await
                .map_err(|_| anyhow!("invalid usage response"))?;
            // Preserve Claude's JSON fields for existing consumers, except Extra Credits.
            data.as_object_mut()
                .ok_or_else(|| anyhow!("invalid usage response"))?
                .remove("extra_usage");
            serde_json::from_value::<crate::models::UsageResponse>(data.clone())
                .map_err(|_| anyhow!("invalid Claude usage windows"))?;
            Ok(data)
        })
    }

    pub fn fetch_codex(&self) -> Result<WindowUsage> {
        let token = self
            .cfg
            .codex_access_token
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| anyhow!("not configured — press e to add credentials"))?;
        let account = self
            .cfg
            .codex_account_id
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| anyhow!("account ID missing — press e to update"))?;
        // Persist only the retry deadline, so SwiftBar's separate --json processes
        // also respect a server Retry-After. No access tokens/cookies are cached.
        let deadline =
            read_backoff(self.backoff_path.as_deref(), account).max(self.codex_retry_until.get());
        let now = chrono::Utc::now().timestamp();
        if deadline > now {
            return Err(anyhow!("rate limited — retry in {}s", deadline - now));
        }
        self.runtime
            .block_on(self.fetch_codex_inner(CODEX_USAGE_URL, token, account))
    }

    async fn fetch_codex_inner(
        &self,
        url: &str,
        token: &str,
        account: &str,
    ) -> Result<WindowUsage> {
        let token = token.trim().strip_prefix("Bearer ").unwrap_or(token.trim());
        let mut request = self
            .client
            .get(url)
            .header("accept", "application/json")
            .header("referer", "https://chatgpt.com/codex/settings/usage")
            .header("authorization", format!("Bearer {token}"))
            .header("ChatGPT-Account-Id", account);
        if let Some(cookie) = &self.cfg.codex_cookie {
            request = request.header("cookie", cookie);
        }
        let response = request
            .send()
            .await
            .map_err(|_| anyhow!("request failed or timed out"))?;
        if response.status().as_u16() == 429 {
            let now = chrono::Utc::now().timestamp();
            let delay = retry_after_seconds(
                response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok()),
                now,
            );
            let deadline = now.saturating_add(delay);
            self.codex_retry_until.set(deadline);
            write_backoff(self.backoff_path.as_deref(), account, deadline);
            return Err(anyhow!("rate limited — retry in {delay}s"));
        }
        check_status(&response)?;
        response
            .json::<CodexResponse>()
            .await
            .map_err(|_| anyhow!("invalid Codex usage response"))?
            .weekly()
    }
}

fn check_status(response: &Response) -> Result<()> {
    match response.status().as_u16() {
        200..=299 => Ok(()),
        code @ (401 | 403) => Err(anyhow!("auth {code} — press e to update credentials")),
        code => Err(anyhow!("http {code}")),
    }
}

fn retry_after_seconds(value: Option<&str>, now: i64) -> i64 {
    value
        .and_then(|s| {
            s.parse::<i64>().ok().or_else(|| {
                chrono::DateTime::parse_from_rfc2822(s)
                    .ok()
                    .map(|t| t.timestamp().saturating_sub(now))
            })
        })
        .unwrap_or(30)
        .max(2)
}

fn read_backoff(path: Option<&std::path::Path>, account: &str) -> i64 {
    let read = || -> Option<i64> {
        let data: Value = serde_json::from_slice(&std::fs::read(path?).ok()?).ok()?;
        (data["account_id"].as_str()? == account)
            .then(|| data["until"].as_i64())
            .flatten()
    };
    read().unwrap_or(0)
}

fn write_backoff(path: Option<&std::path::Path>, account: &str, until: i64) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)
        {
            let _ = write!(
                file,
                "{}",
                serde_json::json!({"account_id": account, "until": until})
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn retry_after_supports_seconds_dates_and_missing_headers() {
        assert_eq!(retry_after_seconds(Some("120"), 0), 120);
        assert_eq!(
            retry_after_seconds(Some("Thu, 01 Jan 1970 00:02:00 GMT"), 0),
            120
        );
        assert_eq!(retry_after_seconds(Some("bad"), 0), 30);
        assert_eq!(retry_after_seconds(Some("-5"), 0), 2);
    }

    fn serve(status: &str, body: &str) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/usage", listener.local_addr().unwrap());
        let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 1024];
            loop {
                let n = stream.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..n]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8(request).unwrap().to_lowercase()
        });
        (url, handle)
    }

    #[test]
    fn direct_request_sends_codex_headers_and_parses_weekly() {
        let (url, server) = serve(
            "200 OK",
            r#"{"rate_limit":{"primary_window":{"used_percent":13,"limit_window_seconds":604800,"reset_at":2000000000},"secondary_window":null}}"#,
        );
        let api = ApiClient::new(Config {
            codex_cookie: Some("cf_clearance=test".into()),
            ..Default::default()
        })
        .unwrap();
        let weekly = api
            .runtime
            .block_on(api.fetch_codex_inner(&url, "Bearer test-token", "workspace-test"))
            .unwrap();
        assert_eq!(weekly.utilization, 13.0);
        let request = server.join().unwrap();
        assert!(request.contains("authorization: bearer test-token\r\n"));
        assert!(request.contains("chatgpt-account-id: workspace-test\r\n"));
        assert!(request.contains("cookie: cf_clearance=test\r\n"));
        assert!(!request.contains("sessionkey"));
    }

    #[test]
    fn error_response_body_is_not_exposed() {
        let (url, server) = serve("401 Unauthorized", "secret-body-must-not-be-shown");
        let api = ApiClient::new(Config::default()).unwrap();
        let error = api
            .runtime
            .block_on(api.fetch_codex_inner(&url, "token", "account"))
            .unwrap_err()
            .to_string();
        server.join().unwrap();
        assert!(error.contains("auth 401"));
        assert!(!error.contains("secret"));
    }
    #[test]
    fn rate_limit_deadline_survives_a_new_client() {
        let (url, server) = serve("429 Too Many Requests\r\nRetry-After: 120", "{}");
        let path = std::env::temp_dir().join(format!(
            "codex-backoff-test-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let cfg = Config {
            codex_access_token: Some("test-token".into()),
            codex_account_id: Some("test-account".into()),
            ..Default::default()
        };
        let mut api = ApiClient::new(cfg.clone()).unwrap();
        api.backoff_path = Some(path.clone());
        let error = api
            .runtime
            .block_on(api.fetch_codex_inner(&url, "test-token", "test-account"))
            .unwrap_err();
        server.join().unwrap();
        assert!(error.to_string().contains("retry in 120s"));
        assert!(api.codex_retry_until.get() > chrono::Utc::now().timestamp());
        let mut next = ApiClient::new(cfg).unwrap();
        next.backoff_path = Some(path.clone());
        // This returns before any external request: a new --json process observes the same deadline.
        assert!(next
            .fetch_codex()
            .unwrap_err()
            .to_string()
            .contains("rate limited"));
        assert_eq!(read_backoff(Some(&path), "different-account"), 0);
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains("test-token"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn credentials_are_not_followed_through_redirects() {
        let (url, server) = serve(
            "302 Found\r\nLocation: http://127.0.0.1:1/should-not-be-contacted",
            "{}",
        );
        let api = ApiClient::new(Config::default()).unwrap();
        let error = api
            .runtime
            .block_on(api.fetch_codex_inner(&url, "token", "account"))
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(error.to_string(), "http 302");
    }
}
