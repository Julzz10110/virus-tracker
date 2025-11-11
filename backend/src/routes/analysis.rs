use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::{Client, StatusCode};
use std::time::Instant;

use crate::{
    auth::{require_role, AuthenticatedUser},
    errors::{ApiError, AppError, AppResult},
    models::{
        AnalysisRequest, AnalysisResponse, MutationAnalysis, PhylogenyResponse, ReferenceResponse,
        UpdateReferenceRequest, VariantQueryParams,
    },
    services::analysis::MutationAnalysisService,
    state::AppState,
};

pub async fn analyze_sequences(
    State(_state): State<AppState>,
    Json(request): Json<AnalysisRequest>,
) -> AppResult<Json<AnalysisResponse>> {
    let start = Instant::now();

    let mutations = MutationAnalysisService::detect_for_sequences(&request);
    let computation_time = start.elapsed().as_secs_f64();

    Ok(Json(AnalysisResponse {
        mutations,
        computation_time,
    }))
}

pub async fn mutation_frequency(
    State(state): State<AppState>,
    Query(params): Query<VariantQueryParams>,
) -> AppResult<Json<Vec<MutationAnalysis>>> {
    let reference = state.get_reference_sequence().await?;
    if reference.is_empty() {
        return Err(AppError::from("Reference sequence is not configured"));
    }

    let variants = state.storage.list_variants(&params).await?;
    let sequences: Vec<String> = variants
        .into_iter()
        .map(|variant| variant.sequence)
        .collect();

    if sequences.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let analysis = MutationAnalysisService::calculate_mutation_frequency(&sequences, &reference);

    Ok(Json(analysis))
}

pub async fn get_reference_sequence(
    State(state): State<AppState>,
) -> AppResult<Json<ReferenceResponse>> {
    let reference = state.get_reference_sequence().await?;
    Ok(Json(ReferenceResponse { reference }))
}

pub async fn set_reference_sequence(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<UpdateReferenceRequest>,
) -> AppResult<Json<ReferenceResponse>> {
    require_role(&user, &["admin"])?;

    state
        .set_reference_sequence(payload.reference.clone())
        .await?;
    Ok(Json(ReferenceResponse {
        reference: payload.reference,
    }))
}

pub async fn phylogeny_summary(
    State(state): State<AppState>,
    Query(params): Query<VariantQueryParams>,
) -> Result<Json<PhylogenyResponse>, ApiError> {
    let client = Client::new();
    let url = format!("{}/api/backend/phylogeny", state.bio_processor_url());
    let mut request = client.get(&url);

    let mut query_params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref lineage) = params.lineage {
        query_params.push(("lineage", lineage.as_str()));
    }
    if let Some(ref location) = params.location {
        query_params.push(("location", location.as_str()));
    }
    if !query_params.is_empty() {
        request = request.query(&query_params);
    }

    let response = request
        .send()
        .await
        .map_err(|err| ApiError::InternalServerError(err.to_string()))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::NotFound("No variants available".into()));
    }

    if !response.status().is_success() {
        return Err(ApiError::InternalServerError(format!(
            "bio_processor returned status {}",
            response.status()
        )));
    }

    let payload = response
        .json::<PhylogenyResponse>()
        .await
        .map_err(|err| ApiError::InternalServerError(err.to_string()))?;

    Ok(Json(payload))
}
