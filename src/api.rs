use anyhow::{anyhow, Result};
use rquest::Client;
use rquest_util::Emulation;
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::models::{AggregatedUsage, UsageResponse};

pub fn fetch_raw_json(cfg: &Config) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let client = Client::builder()
            .emulation(Emulation::Chrome136)
            .timeout(std::time::Duration::from_secs(8))
            .build()?;
        let mut cookie = format!(
            "sessionKey={}; cf_clearance={}",
            cfg.session_key, cfg.cf_clearance
        );
        if let Some(bm) = &cfg.cf_bm {
            cookie.push_str("; __cf_bm=");
            cookie.push_str(bm);
        }
        let resp = client
            .get(format!(
                "https://claude.ai/api/organizations/{}/usage",
                cfg.org_id
            ))
            .header("accept", "application/json")
            .header("referer", "https://claude.ai/settings/usage")
            .header("cookie", cookie)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("http {}", status.as_u16()));
        }
        Ok(resp.text().await?)
    })
}

pub struct ApiClient {
    client: Client,
    runtime: Runtime,
    cfg: Config,
}

impl ApiClient {
    pub fn new(cfg: Config) -> Result<Self> {
        let client = Client::builder()
            .emulation(Emulation::Chrome136)
            .timeout(std::time::Duration::from_secs(8))
            .build()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(ApiClient { client, runtime, cfg })
    }

    pub fn fetch_usage(&self) -> Result<AggregatedUsage> {
        self.runtime.block_on(self.fetch_inner())
    }

    async fn fetch_inner(&self) -> Result<AggregatedUsage> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/usage",
            self.cfg.org_id
        );

        let mut cookie = format!(
            "sessionKey={}; cf_clearance={}",
            self.cfg.session_key, self.cfg.cf_clearance
        );
        if let Some(bm) = &self.cfg.cf_bm {
            cookie.push_str("; __cf_bm=");
            cookie.push_str(bm);
        }

        let resp = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("referer", "https://claude.ai/settings/usage")
            .header("cookie", cookie)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(anyhow!(
                    "auth: {} — cookies likely expired. Press 'e' to update.",
                    status.as_u16()
                ));
            }
            return Err(anyhow!("http {}", status.as_u16()));
        }

        let data: UsageResponse = resp.json().await?;

        let mut aggregated = AggregatedUsage {
            five_hour_util: data.five_hour.utilization,
            five_hour_resets_at: data.five_hour.resets_at,
            seven_day_util: data.seven_day.utilization,
            seven_day_resets_at: data.seven_day.resets_at,
            ..Default::default()
        };

        if let Some(extra) = data.extra_usage {
            aggregated.extra_enabled = extra.is_enabled;
            aggregated.extra_used = extra.used_credits;
            aggregated.extra_limit = extra.monthly_limit;
            aggregated.extra_util = extra.utilization;
            aggregated.currency = extra.currency;
        }

        Ok(aggregated)
    }
}
