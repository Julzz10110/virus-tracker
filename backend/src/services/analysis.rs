use crate::models::{AnalysisRequest, Mutation, MutationAnalysis};
use rayon::prelude::*;
use std::collections::HashMap;

pub struct MutationAnalysisService;

impl MutationAnalysisService {
    pub fn detect_mutations(reference: &str, sequence: &str) -> Vec<Mutation> {
        let mut mutations = Vec::new();

        let min_len = reference.len().min(sequence.len());
        for (i, (ref_base, seq_base)) in reference
            .chars()
            .zip(sequence.chars())
            .enumerate()
            .take(min_len)
        {
            if ref_base != seq_base && seq_base != '-' && ref_base != '-' {
                mutations.push(Mutation {
                    gene: determine_gene(i as u32),
                    position: i as u32,
                    reference: ref_base,
                    mutation: seq_base,
                });
            }
        }

        mutations
    }

    pub fn detect_for_sequences(request: &AnalysisRequest) -> Vec<Vec<Mutation>> {
        request
            .sequences
            .par_iter()
            .map(|sequence| Self::detect_mutations(&request.reference, sequence))
            .collect()
    }

    pub fn calculate_mutation_frequency(
        sequences: &[String],
        reference: &str,
    ) -> Vec<MutationAnalysis> {
        let mut position_counts: HashMap<u32, HashMap<char, u32>> = HashMap::new();
        let total_sequences = sequences.len();

        for sequence in sequences {
            for (i, (ref_base, seq_base)) in reference.chars().zip(sequence.chars()).enumerate() {
                if ref_base != seq_base {
                    let position = i as u32;
                    let entry = position_counts.entry(position).or_insert_with(HashMap::new);
                    *entry.entry(seq_base).or_insert(0) += 1;
                }
            }
        }

        position_counts
            .par_iter()
            .map(|(&position, mutations)| {
                let total_mutations_at_pos: u32 = mutations.values().sum();
                let frequency = total_mutations_at_pos as f64 / total_sequences as f64;

                let entropy = mutations
                    .values()
                    .map(|&count| {
                        let prob = count as f64 / total_mutations_at_pos as f64;
                        -prob * prob.log2()
                    })
                    .sum();

                let (mutation, _) = mutations.iter().max_by_key(|(_, &count)| count).unwrap();

                MutationAnalysis {
                    position,
                    reference: reference.chars().nth(position as usize).unwrap_or_default(),
                    mutation: *mutation,
                    frequency,
                    entropy,
                    gene: determine_gene(position),
                }
            })
            .collect()
    }
}

fn determine_gene(position: u32) -> String {
    match position {
        0..=10000 => "ORF1a".to_string(),
        10001..=20000 => "ORF1b".to_string(),
        20001..=22000 => "Spike".to_string(),
        22001..=23000 => "ORF3a".to_string(),
        23001..=24000 => "E".to_string(),
        24001..=25000 => "M".to_string(),
        25001..=26000 => "ORF6".to_string(),
        26001..=27000 => "ORF7a".to_string(),
        27001..=28000 => "ORF8".to_string(),
        _ => "N".to_string(),
    }
}
