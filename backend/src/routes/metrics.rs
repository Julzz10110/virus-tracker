use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    models::{GrowthMetric, GrowthSummary},
    AppState,
};

#[derive(Debug, Deserialize)]
struct GrowthMetricsQuery {
    limit: Option<usize>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/api/metrics/growth", get(list_growth_metrics))
        .route("/api/metrics/growth/live", get(live_growth_metrics))
}

async fn prometheus_metrics(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let body = state.metrics_handle().render();
    Ok(([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body))
}

async fn list_growth_metrics(
    State(state): State<AppState>,
    Query(params): Query<GrowthMetricsQuery>,
) -> AppResult<Json<Vec<GrowthMetric>>> {
    let limit = params.limit.unwrap_or(30);
    let metrics = state.storage.list_growth_metrics(limit).await?;
    Ok(Json(metrics))
}

async fn live_growth_metrics(State(state): State<AppState>) -> AppResult<Json<GrowthSummary>> {
    let client = Client::new();
    let url = format!(
        "{}/api/backend/epidemiology/growth",
        state.bio_processor_url()
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| AppError::from(err.to_string()))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(Json(GrowthSummary {
            total_count: 0,
            growth_rate: None,
            reproduction_number: None,
            doubling_time: None,
        }));
    }

    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "bio_processor returned status {}",
            response.status()
        )));
    }

    let payload = response
        .json::<GrowthSummary>()
        .await
        .map_err(|err| AppError::from(err.to_string()))?;

    Ok(Json(payload))
}
