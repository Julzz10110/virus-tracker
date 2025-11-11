use metrics_exporter_prometheus::PrometheusBuilder;
use std::env;
use std::path::Path;
use tower_http::trace::TraceLayer;
use virus_tracker_axum::{routes, services::tasks, storage::Storage, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/virus_tracker.db".to_string());

    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let storage = Storage::connect(&database_url).await?;
    storage.migrate().await?;

    let api_key = env::var("API_KEY").ok().filter(|key| !key.is_empty());
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required for authentication");
    let token_ttl_hours = env::var("TOKEN_TTL_HOURS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(24);
    let growth_alert_threshold = env::var("GROWTH_ALERT_THRESHOLD")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(1.2);
    let bio_processor_url =
        env::var("BIO_PROCESSOR_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

    let state = AppState::new(
        storage,
        api_key,
        jwt_secret,
        token_ttl_hours,
        prometheus_handle.clone(),
        growth_alert_threshold,
        bio_processor_url.clone(),
    );

    tasks::spawn_background_jobs(state.clone());

    let app = routes::create_router(state).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("🚀 Axum server running on http://0.0.0.0:8080");

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
