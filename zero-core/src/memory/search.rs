use std::cmp::Ordering;
use std::collections::HashMap;

use super::{MemoryEntry, MemoryError};

/// Lightweight in-memory search index entry pairing a stored memory entry with its vector embedding.
#[derive(Clone, Debug)]
pub struct MemoryIndexedEntry {
    pub entry: MemoryEntry,
    pub embedding: Vec<f32>,
}

/// Simple in-memory search index with optional vector embeddings and metadata-aware filtering.
pub struct MemorySearchIndex {
    // Cached indexed entries for faster repeated searches
    pub indexed: Vec<MemoryIndexedEntry>,
}

impl MemorySearchIndex {
    pub fn new() -> Self {
        Self {
            indexed: Vec::new(),
        }
    }

    /// Rebuild the index from a slice of memory entries. This will re-embed the content
    /// for vector-based search. In a real system this would be incremental and thread-safe.
    pub fn index_entries(&mut self, entries: &[MemoryEntry]) {
        self.indexed.clear();
        for e in entries {
            // Attempt to extract textual content; fall back to an empty string if unavailable.
            let text = e.content.clone().unwrap_or_else(|| String::new());
            let embedding = embed_text(&text);
            self.indexed.push(MemoryIndexedEntry {
                entry: e.clone(),
                embedding,
            });
        }
    }

    /// Perform a search over the index given a raw query string. This combines a lightweight
    /// vector similarity signal with a simple full-text signal and a metadata filter step.
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        filters: &HashMap<String, String>,
    ) -> Vec<MemoryEntry> {
        // Parse query for text and potential structured filters
        let (text_query, _text_filters) = parse_query(query);

        // Embed the query string for vector scoring
        let query_emb = embed_text(&text_query);

        // Score each indexed entry
        let mut scored: Vec<(f32, MemoryEntry)> = Vec::with_capacity(self.indexed.len());
        for idx in &self.indexed {
            // Metadata filtering: fail fast if metadata doesn't match
            if !filters_match(&idx.entry, filters) {
                continue;
        for idx in &self.indexed {
            // Metadata filtering: fail fast if metadata doesn't match
            if !filters_match(&idx.entry, filters) {
                continue;
            }

            let content_str = idx.entry.value.to_lowercase();
            let mut text_score = 0.0f32;
            if !text_query.is_empty() {
                if content_str.contains(&text_query.to_lowercase()) {
                    text_score += 0.6;
                }
                for w in text_query.split_whitespace() {
                    if !w.is_empty() && content_str.contains(&w.to_lowercase()) {
                        text_score += 0.1;
                    }
                }
            }

            let final_score = vec_score * 0.6 + text_score * 0.4;
            scored.push((final_score, idx.entry.clone()));
        }
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    scored.into_iter().take(limit).map(|(_s, e)| e).collect()
}

            let content_str = idx.entry.value.to_lowercase();

            // Full-text match heuristic: simple substring in content
            let content_str = idx.entry.value.to_lowercase();
                .entry
                .content
                .as_ref()
                .unwrap_or(&String::new())
                .to_lowercase();
            let mut text_score = 0.0f32;
            if !text_query.is_empty() {
                if content_str.contains(&text_query.to_lowercase()) {
                    text_score += 0.6;
                }
                // rough keyword match for added surface score
                let q_words: Vec<&str> = text_query.split_whitespace().collect();
                for w in q_words {
                    if !w.is_empty() && content_str.contains(&w.to_lowercase()) {
                        text_score += 0.1;
                    }
                }
            }

            // Metadata-driven score (favor entries that match known keys/values exactly)
            let mut meta_score = 0.0f32;
            for (k, v) in filters {
                if let Some(val) = idx.entry.metadata.as_ref().and_then(|m| m.get(k)) {
                    if val == v {
                        meta_score += 0.15;
                    }
                }

            let final_score = vec_score * 0.6 + text_score * 0.4

            let final_score = vec_score * 0.5 + text_score * 0.35 + meta_score;
            scored.push((final_score, idx.entry.clone()));
        }

        // Sort by score descending and return up to limit results
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        scored.into_iter().take(limit).map(|(_s, e)| e).collect()
    }
}

/// Very small, deterministic pseudo-embedding for a given string. This is a placeholder for a real embedding model.
fn embed_text(text: &str) -> Vec<f32> {
    let mut v = vec![0f32; 16];
    for (i, token) in text.split_whitespace().enumerate() {
        let idx = i % v.len();
        v[idx] += (token.len() as f32) * 0.1
            + (token.as_bytes().first().copied().unwrap_or(0) as f32) * 0.01;
    }
    // normalize
    let mut norm = 0f32;
    for x in &v {
        norm += x * x;
    }
    if norm > 0.0 {
        let inv = 1f32 / norm.sqrt();
        for x in &mut v {
            *x *= inv;
        }
    }
    v
    dot / (na.sqrt() * nb.sqrt())
}

fn filters_match

/// Compute cosine similarity between two equal-length vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0f32;
    let mut na = 0f32;
    let mut nb = 0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Lightweight helpers to parse query strings into a text portion and a map of metadata filters.
fn parse_query(query: &str) -> (String, HashMap<String, String>) {
    let mut text = String::new();
    let mut meta = HashMap::new();
    for part in query.split_whitespace() {
        if let Some(pos) = part.find(':') {
            let key = part[..pos].to_string();
            let val = part[pos + 1..].to_string();
            if !key.is_empty() && !val.is_empty() {
                meta.insert(key, val);
            }
        } else {
            if !part.is_empty() {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(part);
            }
        }
    }
    (text, meta)
}

fn filters_match(entry: &MemoryEntry, filters: &HashMap<String, String>) -> bool {
    if filters.is_empty() {
        return true;
    }
    if let Some(meta) = entry.metadata.as_ref() {
        for (k, v) in filters {
            if let Some(val) = meta.get(k) {
                if val != v {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// Lightweight, non-persistent search across a static slice of memory entries.
/// This provides a path for the trait default implementation to leverage without
/// requiring access to internal index state.
pub fn search_entries(entries: &[MemoryEntry], query: &str) -> Vec<MemoryEntry> {
    if entries.is_empty() || query.is_empty() {
        return entries.to_vec();
    }
    // Parse query to extract text and potential metadata filters
    let (text, filters) = parse_query(query);
    let query_emb = embed_text(&text);
    let mut scored: Vec<(f32, MemoryEntry)> = Vec::with_capacity(entries.len());
    for e in entries {
        // Apply metadata filters if present
        if !filters_match(e, &filters) {
            continue;
        }

        let emb = embed_text(e.value.as_str());
        let vec_score = cosine_similarity(&emb, &query_emb);
        let content = e.value.to_lowercase();
        let mut text_score = 0.0f32;
        if content.contains(&query.to_lowercase()) {
            text_score += 0.6;
        }
        for w in query.split_whitespace() {
            if !w.is_empty() && content.contains(&w.to_lowercase()) {
                text_score += 0.05;
            }
        }
        let final_score = vec_score * 0.6 + text_score * 0.4;
        scored.push((final_score, e.clone()));
    }
}
        if !filters_match(e, &filters) {
            continue;
        }
        let emb = embed_text(e.value.as_str());
        let vec_score = cosine_similarity(&emb, &query_emb);
        let content = e.value.to_lowercase();
        let mut text_score = 0.0f32;
        if content.contains(&query.to_lowercase()) {
            text_score += 0.6;
        }
        // simple token match
        for w in query.split_whitespace() {
            if !w.is_empty() && content.contains(&w.to_lowercase()) {
                text_score += 0.05;
            }
        }
        let final_score = vec_score * 0.6 + text_score * 0.4;
        scored.push((final_score, e.clone()));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_s, e)| e).collect()
}
