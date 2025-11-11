use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    auth::{require_role, AuthenticatedUser},
    errors::AppResult,
    models::{UploadVariantRequest, VariantQueryParams, VariantStats, VirusVariant},
    services::analysis::MutationAnalysisService,
    state::AppState,
};

pub async fn get_variants(
    State(state): State<AppState>,
    Query(params): Query<VariantQueryParams>,
) -> AppResult<Json<Vec<VirusVariant>>> {
    let variants = state.storage.list_variants(&params).await?;
    Ok(Json(variants))
}

pub async fn get_variant_stats(State(state): State<AppState>) -> AppResult<Json<VariantStats>> {
    let stats = state.storage.get_stats().await?;
    Ok(Json(stats))
}

pub async fn upload_variant(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<UploadVariantRequest>,
) -> AppResult<(StatusCode, Json<VirusVariant>)> {
    require_role(&user, &["admin", "uploader"])?;

    let reference_sequence = state.get_reference_sequence().await?;

    let variant = VirusVariant {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        lineage: request.lineage,
        mutations: MutationAnalysisService::detect_mutations(
            &reference_sequence,
            &request.sequence,
        ),
        location: request.location,
        date: Utc::now().to_rfc3339(),
        sequence: request.sequence,
    };

    state
        .storage
        .insert_variant_with_mutations(&variant)
        .await?;

    Ok((StatusCode::CREATED, Json(variant)))
}
