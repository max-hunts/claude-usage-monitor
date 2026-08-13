use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct UsageResponse {
    pub five_hour: WindowUsage,
    pub seven_day: WindowUsage,
    #[serde(default)]
    pub limits: Vec<LimitEntry>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WindowUsage {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

/// One entry of the API's `limits` array. Model-scoped weekly limits (e.g. the
/// Fable-only weekly cap) are only reported here, not as a top-level window.
#[derive(Debug, Deserialize, Clone)]
pub struct LimitEntry {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub percent: Option<f64>,
    #[serde(default)]
    pub resets_at: Option<String>,
    #[serde(default)]
    pub scope: Option<LimitScope>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitScope {
    #[serde(default)]
    pub model: Option<ScopeModel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScopeModel {
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: f64,
    pub used_credits: f64,
    pub utilization: f64,
    pub currency: String,
}

/// A weekly window scoped to a single model, labeled by that model's name.
#[derive(Debug, Clone)]
pub struct ScopedWindow {
    pub label: String,
    pub util: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AggregatedUsage {
    pub five_hour_util: f64,
    pub five_hour_resets_at: Option<String>,
    pub seven_day_util: f64,
    pub seven_day_resets_at: Option<String>,
    pub scoped_weekly: Vec<ScopedWindow>,
    pub extra_enabled: bool,
    pub extra_used: f64,
    pub extra_limit: f64,
    pub extra_util: f64,
    pub currency: String,
}
