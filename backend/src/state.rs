use crate::storage::Storage;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    fallback_reference: Arc<String>,
    api_key: Option<Arc<String>>,
    jwt_secret: Arc<String>,
    token_ttl_hours: i64,
    metrics_handle: Arc<PrometheusHandle>,
    growth_alert_threshold: f64,
    bio_processor_url: Arc<String>,
}

impl AppState {
    pub fn new(
        storage: Storage,
        api_key: Option<String>,
        jwt_secret: String,
        token_ttl_hours: i64,
        metrics_handle: PrometheusHandle,
        growth_alert_threshold: f64,
        bio_processor_url: String,
    ) -> Self {
        Self {
            storage,
            fallback_reference: Arc::new(Self::default_reference()),
            api_key: api_key.map(Arc::new),
            jwt_secret: Arc::new(jwt_secret),
            token_ttl_hours,
            metrics_handle: Arc::new(metrics_handle),
            growth_alert_threshold,
            bio_processor_url: Arc::new(bio_processor_url),
        }
    }

    pub fn with_fallback(
        storage: Storage,
        reference: impl Into<String>,
        api_key: Option<String>,
        jwt_secret: String,
        token_ttl_hours: i64,
        metrics_handle: PrometheusHandle,
        growth_alert_threshold: f64,
        bio_processor_url: String,
    ) -> Self {
        Self {
            storage,
            fallback_reference: Arc::new(reference.into()),
            api_key: api_key.map(Arc::new),
            jwt_secret: Arc::new(jwt_secret),
            token_ttl_hours,
            metrics_handle: Arc::new(metrics_handle),
            growth_alert_threshold,
            bio_processor_url: Arc::new(bio_processor_url),
        }
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref().map(|arc| arc.as_str())
    }

    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub fn token_ttl_hours(&self) -> i64 {
        self.token_ttl_hours
    }

    pub fn metrics_handle(&self) -> Arc<PrometheusHandle> {
        Arc::clone(&self.metrics_handle)
    }

    pub fn growth_alert_threshold(&self) -> f64 {
        self.growth_alert_threshold
    }

    pub fn bio_processor_url(&self) -> &str {
        self.bio_processor_url.as_str()
    }

    pub async fn set_reference_sequence(
        &self,
        reference: impl Into<String>,
    ) -> Result<(), sqlx::Error> {
        self.storage.set_reference_sequence(&reference.into()).await
    }

    pub async fn get_reference_sequence(&self) -> Result<String, sqlx::Error> {
        match self.storage.get_reference_sequence().await? {
            Some(reference) => Ok(reference),
            None => Ok((*self.fallback_reference).clone()),
        }
    }

    fn default_reference() -> String {
        "REFERENCE_SEQUENCE".to_string()
    }
}
