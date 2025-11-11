use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

use virus_tracker_axum::{routes::create_router, storage::Storage, AppState};

static PROMETHEUS: OnceLock<PrometheusHandle> = OnceLock::new();

async fn setup_app_state() -> AppState {
    let storage = Storage::connect("sqlite::memory:?cache=shared")
        .await
        .expect("connect sqlite memory");
    storage.migrate().await.expect("run migrations");

    let handle = PROMETHEUS.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("install recorder")
    });

    AppState::with_fallback(
        storage,
        "ACGTACGT",
        Some("test-key".to_string()),
        "test-secret".to_string(),
        1,
        handle.clone(),
        1.2,
        "http://localhost:8000".to_string(),
    )
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let state = setup_app_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(std::str::from_utf8(&body).unwrap(), "OK");
}

#[tokio::test]
async fn can_upload_variant_and_retrieve_stats() {
    let state = setup_app_state().await;
    let app = create_router(state);

    let payload = json!({
        "name": "Sample Variant",
        "lineage": "B.1",
        "location": "Testland",
        "sequence": "ACGTTCGT"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload")
                .header("content-type", "application/json")
                .header("x-api-key", "test-key")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/variants")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let variants: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(variants.as_array().unwrap().len(), 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/variants/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let stats: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["total_variants"], 1);
    assert_eq!(stats["unique_locations"], 1);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analysis/frequency")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let frequency: Value = serde_json::from_slice(&body).unwrap();
    assert!(frequency.as_array().unwrap().len() >= 1);
}
