use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirusVariant {
    pub id: String,
    pub name: String,
    pub lineage: String,
    pub mutations: Vec<Mutation>,
    pub location: String,
    pub date: String,
    pub sequence: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mutation {
    pub gene: String,
    pub position: u32,
    pub reference: char,
    pub mutation: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub sequences: Vec<String>,
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateReferenceRequest {
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceResponse {
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub mutations: Vec<Vec<Mutation>>,
    pub computation_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadVariantRequest {
    pub name: String,
    pub lineage: String,
    pub location: String,
    pub sequence: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutationAnalysis {
    pub position: u32,
    pub reference: char,
    pub mutation: char,
    pub frequency: f64,
    pub entropy: f64,
    pub gene: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantStats {
    pub total_variants: usize,
    pub unique_locations: usize,
    pub total_mutations: usize,
    pub average_mutations_per_variant: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantQueryParams {
    pub lineage: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrowthMetric {
    pub collected_at: String,
    pub total_variants: usize,
    pub growth_rate: Option<f64>,
    pub reproduction_number: Option<f64>,
    pub doubling_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrowthSummary {
    pub total_count: usize,
    pub growth_rate: Option<f64>,
    pub reproduction_number: Option<f64>,
    pub doubling_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentSummaryInfo {
    pub consensus: String,
    pub length: usize,
    pub record_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhylogenyNode {
    pub name: Option<String>,
    pub branch_length: Option<f64>,
    #[serde(default)]
    pub children: Vec<PhylogenyNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhylogenyResponse {
    pub tree: PhylogenyNode,
    #[serde(default)]
    pub alignment: Option<AlignmentSummaryInfo>,
    #[serde(default)]
    pub count: usize,
}

impl VariantStats {
    pub fn from_variants(variants: &[VirusVariant]) -> Self {
        let total_variants = variants.len();
        let unique_locations = variants
            .iter()
            .map(|variant| variant.location.clone())
            .collect::<HashSet<_>>()
            .len();
        let total_mutations: usize = variants.iter().map(|variant| variant.mutations.len()).sum();
        let average_mutations_per_variant = if total_variants > 0 {
            total_mutations as f64 / total_variants as f64
        } else {
            0.0
        };

        Self {
            total_variants,
            unique_locations,
            total_mutations,
            average_mutations_per_variant,
        }
    }
}
