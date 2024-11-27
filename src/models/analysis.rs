use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiAnalysis {
    pub auth_type: Option<String>,
    pub rate_limit: Option<u32>,
    pub cache_status: CacheAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CacheAnalysis {
    pub cacheable: bool,
    pub suggested_ttl: Option<u32>,
    pub reason: String,
}