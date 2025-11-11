use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::state::AppState;

pub mod analysis;
pub mod auth;
pub mod health;
pub mod metrics;
pub mod variants;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/analyze", post(analysis::analyze_sequences))
        .route("/api/analysis/frequency", get(analysis::mutation_frequency))
        .route("/api/analysis/phylogeny", get(analysis::phylogeny_summary))
        .route("/api/variants", get(variants::get_variants))
        .route("/api/variants/stats", get(variants::get_variant_stats))
        .route("/api/upload", post(variants::upload_variant))
        .route(
            "/api/reference",
            get(analysis::get_reference_sequence).post(analysis::set_reference_sequence),
        )
        .merge(auth::routes())
        .merge(metrics::routes())
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}
