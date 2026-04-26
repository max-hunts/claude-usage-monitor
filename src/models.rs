use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct UsageResponse {
    pub five_hour: WindowUsage,
    pub seven_day: WindowUsage,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WindowUsage {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: f64,
    pub used_credits: f64,
    pub utilization: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Default)]
pub struct AggregatedUsage {
    pub five_hour_util: f64,
    pub five_hour_resets_at: Option<String>,
    pub seven_day_util: f64,
    pub seven_day_resets_at: Option<String>,
    pub extra_enabled: bool,
    pub extra_used: f64,
    pub extra_limit: f64,
    pub extra_util: f64,
    pub currency: String,
}
