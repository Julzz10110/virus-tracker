use std::time::Duration;

use anyhow::{self, Result};
use chrono::Utc;
use metrics::{counter, gauge};
use reqwest::Client;
use tokio::time::interval;
use tracing::{info, warn};

use crate::{
    models::{GrowthMetric, GrowthSummary},
    AppState,
};

pub fn spawn_background_jobs(state: AppState) {
    tokio::spawn(async move {
        let client = Client::new();
        let mut ticker = interval(Duration::from_secs(3600));
        loop {
            ticker.tick().await;
            if let Err(err) = collect_growth_metrics(&state, &client).await {
                warn!("Failed to collect growth metrics: {err}");
            }
        }
    });
}

async fn collect_growth_metrics(state: &AppState, client: &Client) -> Result<()> {
    let url = format!(
        "{}/api/backend/epidemiology/growth",
        state.bio_processor_url()
    );
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("bio_processor returned status {}", response.status());
    }

    let payload: GrowthSummary = response.json().await?;

    let metric = GrowthMetric {
        collected_at: Utc::now().to_rfc3339(),
        total_variants: payload.total_count,
        growth_rate: payload.growth_rate,
        reproduction_number: payload.reproduction_number,
        doubling_time: payload.doubling_time,
    };

    state.storage.insert_growth_metric(&metric).await?;

    counter!("growth_metrics_collected_total", 1);
    gauge!("growth_total_variants", metric.total_variants as f64);
    if let Some(rate) = metric.growth_rate {
        gauge!("growth_rate_latest", rate);
    }
    if let Some(reproduction) = metric.reproduction_number {
        gauge!("reproduction_number_latest", reproduction);
        if reproduction > state.growth_alert_threshold() {
            warn!(
                target: "alerts",
                reproduction,
                threshold = state.growth_alert_threshold(),
                "Reproduction number exceeded alert threshold"
            );
        }
    }
    if let Some(doubling) = metric.doubling_time {
        gauge!("doubling_time_latest", doubling);
    }

    info!(
        "Stored growth metrics snapshot with {} variants",
        metric.total_variants
    );

    Ok(())
}
