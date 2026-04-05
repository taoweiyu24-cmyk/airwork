//! Reciprocal Rank Fusion (RRF) for combining keyword and vector search results.

use std::collections::HashMap;

use crate::types::{SearchResult, SearchSource};

/// Fuse keyword and vector search results using Reciprocal Rank Fusion.
///
/// The RRF score for a document is the sum over each ranking system of
/// `1 / (k + rank)` where `rank` is 1-based.  A higher fused score means a
/// more relevant document.
///
/// # Arguments
///
/// * `keyword_results` – Results from FTS5 keyword search, best first.
/// * `vector_results`  – Results from vector similarity search, best first.
/// * `k`               – Smoothing constant (typically 60).
pub fn reciprocal_rank_fusion(
    keyword_results: &[SearchResult],
    vector_results: &[SearchResult],
    k: u32,
) -> Vec<SearchResult> {
    // id -> (fused_score, title, snippet)
    let mut scores: HashMap<String, (f64, String, String)> = HashMap::new();

    for (rank, result) in keyword_results.iter().enumerate() {
        let rrf_score = 1.0 / (k as f64 + rank as f64 + 1.0);
        let entry = scores
            .entry(result.id.clone())
            .or_insert_with(|| (0.0, result.title.clone(), result.snippet.clone()));
        entry.0 += rrf_score;
    }

    for (rank, result) in vector_results.iter().enumerate() {
        let rrf_score = 1.0 / (k as f64 + rank as f64 + 1.0);
        let entry = scores
            .entry(result.id.clone())
            .or_insert_with(|| (0.0, result.title.clone(), result.snippet.clone()));
        entry.0 += rrf_score;
    }

    let mut results: Vec<SearchResult> = scores
        .into_iter()
        .map(|(id, (score, title, snippet))| SearchResult {
            id,
            title,
            snippet,
            score,
            source: SearchSource::Hybrid,
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SearchSource;

    fn make_result(id: &str, title: &str, score: f64, source: SearchSource) -> SearchResult {
        SearchResult {
            id: id.to_string(),
            title: title.to_string(),
            snippet: String::new(),
            score,
            source,
        }
    }

    #[test]
    fn empty_inputs_return_empty_output() {
        let result = reciprocal_rank_fusion(&[], &[], 60);
        assert!(result.is_empty());
    }

    #[test]
    fn keyword_only_results_are_fused() {
        let keyword = vec![
            make_result("a", "Alpha", 0.9, SearchSource::Keyword),
            make_result("b", "Beta", 0.5, SearchSource::Keyword),
        ];
        let fused = reciprocal_rank_fusion(&keyword, &[], 60);
        assert_eq!(fused.len(), 2);
        // Best keyword result should score highest.
        assert_eq!(fused[0].id, "a");
        for r in &fused {
            assert!(matches!(r.source, SearchSource::Hybrid));
        }
    }

    #[test]
    fn document_appearing_in_both_lists_scores_higher() {
        let keyword = vec![make_result("shared", "Shared Doc", 0.8, SearchSource::Keyword)];
        let vector = vec![make_result("shared", "Shared Doc", 0.9, SearchSource::Vector)];
        let fused = reciprocal_rank_fusion(&keyword, &vector, 60);
        assert_eq!(fused.len(), 1);
        // Score should be the sum of two RRF contributions.
        let expected = 1.0 / 61.0 + 1.0 / 61.0;
        assert!((fused[0].score - expected).abs() < 1e-10);
    }

    #[test]
    fn results_are_sorted_descending_by_score() {
        let keyword = vec![
            make_result("a", "A", 0.9, SearchSource::Keyword),
            make_result("b", "B", 0.5, SearchSource::Keyword),
        ];
        let vector = vec![make_result("b", "B", 0.6, SearchSource::Vector)];
        let fused = reciprocal_rank_fusion(&keyword, &vector, 60);
        for window in fused.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }
}
