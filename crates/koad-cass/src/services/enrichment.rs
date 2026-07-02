//! Pure enrichment logic: LLM prompt construction, strict-JSON output parsing,
//! and fill-only-empty metadata merging. No I/O — fully unit-testable.
//!
//! Merge policy (spec §3): agent-supplied metadata is never overwritten. A
//! deterministic default written at commit time (salience == confidence) is
//! treated as overwritable. LLM booleans for privacy only escalate (false→true).

use anyhow::{Context, Result};
use koad_proto::cass::v1::{MemoryMetadata, PrivacyMetadata, RetrievalMetadata};
use serde::Deserialize;

/// Marker key inside metadata_json. Presence = already enriched; worker and
/// backfill skip the LLM pass (but still ensure the embedding exists).
pub const ENRICHMENT_KEY: &str = "enrichment";

#[derive(Debug, Deserialize)]
pub struct EnrichmentOutput {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub salience: f32,
    #[serde(default)]
    pub volatility: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub sensitivity: String,
    #[serde(default)]
    pub contains_pii: bool,
    #[serde(default)]
    pub contains_secret: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Build the single-shot enrichment prompt. Output contract is strict JSON.
pub fn build_enrichment_prompt(content: &str) -> String {
    format!(
        "You are a memory metadata annotator for an AI agent memory system. \
Analyze the memory content below and respond with ONLY a single JSON object \
(no prose, no markdown fences) exactly matching this schema:\n\
{{\"summary\": \"one-sentence summary\", \
\"salience\": 0.0, \
\"volatility\": \"stable|mutable|ephemeral\", \
\"audience\": \"self|agent|team|global\", \
\"sensitivity\": \"public|internal|private|secret-adjacent\", \
\"contains_pii\": false, \
\"contains_secret\": false, \
\"tags\": [\"lowercase-kebab-tag\"]}}\n\
Rules: salience is 0.0-1.0 (1.0 = architectural decision, resolved bug, or \
user preference; 0.0 = noise). volatility: stable = identity/conventions, \
mutable = project state, ephemeral = transient status. tags: 1-5 topical \
tags.\n\nMEMORY CONTENT:\n{}",
        content
    )
}

/// Parse the model's reply. Tolerates leading/trailing prose and markdown
/// fences by extracting the first '{' .. last '}' span.
pub fn parse_enrichment_output(raw: &str) -> Result<EnrichmentOutput> {
    let start = raw.find('{').context("no JSON object in model output")?;
    let end = raw.rfind('}').context("no closing brace in model output")?;
    if end < start {
        anyhow::bail!("malformed JSON span in model output");
    }
    let mut out: EnrichmentOutput =
        serde_json::from_str(&raw[start..=end]).context("model output is not valid JSON")?;
    out.salience = out.salience.clamp(0.0, 1.0);
    Ok(out)
}

/// Merge LLM output into metadata. Fill-only-empty; `confidence` identifies
/// the deterministic salience default written by `default_metadata` at commit.
/// Records the enrichment marker (model + tags + raw salience) in metadata_json.
pub fn merge_enrichment(
    md: &mut MemoryMetadata,
    out: &EnrichmentOutput,
    confidence: f32,
    model: &str,
) {
    if md.summary.is_empty() && !out.summary.is_empty() {
        md.summary = out.summary.clone();
    }

    let rt = md.retrieval.get_or_insert_with(RetrievalMetadata::default);
    // salience == confidence is the deterministic default from commit time — overwritable.
    if (rt.salience == 0.0 || (rt.salience - confidence).abs() < f32::EPSILON) && out.salience > 0.0
    {
        rt.salience = out.salience;
    }
    if rt.volatility.is_empty() && !out.volatility.is_empty() {
        rt.volatility = out.volatility.clone();
    }
    if rt.audience.is_empty() && !out.audience.is_empty() {
        rt.audience = out.audience.clone();
    }

    let pv = md.privacy.get_or_insert_with(PrivacyMetadata::default);
    if pv.sensitivity.is_empty() && !out.sensitivity.is_empty() {
        pv.sensitivity = out.sensitivity.clone();
    }
    // Escalate-only: LLM can flag PII/secrets, never clear an agent's flag.
    pv.contains_pii = pv.contains_pii || out.contains_pii;
    pv.contains_secret = pv.contains_secret || out.contains_secret;

    // Marker + tags in the metadata_json escape hatch.
    let mut extra: serde_json::Value =
        serde_json::from_str(&md.metadata_json).unwrap_or_else(|_| serde_json::json!({}));
    if !extra.is_object() {
        extra = serde_json::json!({});
    }
    extra[ENRICHMENT_KEY] = serde_json::json!({
        "model": model,
        "at": chrono::Utc::now().to_rfc3339(),
        "tags": out.tags,
        "llm_salience": out.salience,
    });
    md.metadata_json = extra.to_string();
}

/// True if this metadata already carries the enrichment marker.
pub fn is_enriched(md: &Option<MemoryMetadata>) -> bool {
    md.as_ref()
        .and_then(|m| serde_json::from_str::<serde_json::Value>(&m.metadata_json).ok())
        .map(|v| v.get(ENRICHMENT_KEY).is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_output() -> EnrichmentOutput {
        parse_enrichment_output(
            r#"{"summary": "Qdrant uses cosine distance", "salience": 0.9,
                "volatility": "stable", "audience": "team",
                "sensitivity": "internal", "contains_pii": false,
                "contains_secret": false, "tags": ["qdrant", "vectors"]}"#,
        )
        .unwrap()
    }

    #[test]
    fn parses_json_wrapped_in_prose_and_fences() {
        let raw = "Sure! Here is the JSON:\n```json\n{\"summary\": \"s\", \"salience\": 1.7}\n```";
        let out = parse_enrichment_output(raw).unwrap();
        assert_eq!(out.summary, "s");
        assert_eq!(out.salience, 1.0); // clamped
    }

    #[test]
    fn rejects_output_without_json() {
        assert!(parse_enrichment_output("I cannot help with that.").is_err());
    }

    #[test]
    fn merge_fills_empty_fields_and_sets_marker() {
        let mut md = MemoryMetadata::default();
        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");
        assert_eq!(md.summary, "Qdrant uses cosine distance");
        let rt = md.retrieval.as_ref().unwrap();
        assert_eq!(rt.salience, 0.9);
        assert_eq!(rt.volatility, "stable");
        assert_eq!(rt.audience, "team");
        assert!(is_enriched(&Some(md)));
    }

    #[test]
    fn merge_never_overwrites_agent_supplied_values() {
        let mut md = MemoryMetadata::default();
        md.summary = "agent wrote this".to_string();
        md.retrieval = Some(RetrievalMetadata {
            salience: 0.42, // != confidence 0.8 → agent-supplied, keep
            volatility: "ephemeral".to_string(),
            audience: "self".to_string(),
            ..Default::default()
        });
        md.privacy = Some(PrivacyMetadata {
            contains_pii: true, // escalate-only: stays true
            ..Default::default()
        });

        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");

        assert_eq!(md.summary, "agent wrote this");
        let rt = md.retrieval.as_ref().unwrap();
        assert_eq!(rt.salience, 0.42);
        assert_eq!(rt.volatility, "ephemeral");
        assert_eq!(rt.audience, "self");
        assert!(md.privacy.as_ref().unwrap().contains_pii);
    }

    #[test]
    fn merge_overwrites_deterministic_salience_default() {
        let mut md = MemoryMetadata::default();
        md.retrieval = Some(RetrievalMetadata {
            salience: 0.8, // == confidence → deterministic default, overwritable
            ..Default::default()
        });
        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");
        assert_eq!(md.retrieval.as_ref().unwrap().salience, 0.9);
    }

    #[test]
    fn is_enriched_false_for_fresh_metadata() {
        assert!(!is_enriched(&Some(MemoryMetadata::default())));
        assert!(!is_enriched(&None));
    }
}
