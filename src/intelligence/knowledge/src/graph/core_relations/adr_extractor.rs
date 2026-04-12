//! ADR关系抽取
use crate::knowledge::adr::AdrEntry;
use crate::knowledge::graph::{GraphError, Relation, Result};
use regex::Regex;

pub struct AdrRelationExtractor;

impl AdrRelationExtractor {
    pub fn extract_from_adr(adr: &AdrEntry) -> Result<Vec<Relation>> {
        let mut rels = Vec::new();
        let re1 = Regex::new(r"(?:参见|参考|引用|references?)\s*:?\s*ADR-(\d{3})")
            .map_err(|e| GraphError::InvalidPattern(format!("Invalid regex: {}", e)))?;
        for cap in re1.captures_iter(&adr.content) {
            if let Some(m) = cap.get(1) {
                let tid = format!("ADR-{}", m.as_str());
                if tid != adr.id { rels.push(Relation {
                    id: format!("{}-ref-{}", adr.id, tid), subject: adr.id.clone(),
                    predicate: "references".into(), object: tid, confidence: 0.9,
                    extracted_from: Some(adr.id.clone()), created_at: chrono::Utc::now().timestamp(),
                }); }
            }
        }
        let re2 = Regex::new(r"(?:依赖|使用|requires?)\s*:?\s*(\w+)")
            .map_err(|e| GraphError::InvalidPattern(format!("Invalid regex: {}", e)))?;
        for cap in re2.captures_iter(&adr.content) {
            if let Some(m) = cap.get(1) {
                let d = m.as_str();
                if d != adr.id { rels.push(Relation {
                    id: format!("{}-dep-{}", adr.id, d), subject: adr.id.clone(),
                    predicate: "depends".into(), object: d.into(), confidence: 0.8,
                    extracted_from: Some(adr.id.clone()), created_at: chrono::Utc::now().timestamp(),
                }); }
            }
        }
        Ok(rels)
    }

    pub fn extract_batch(adrs: &[AdrEntry], prog: impl Fn(usize, usize)) -> Result<Vec<Relation>> {
        let mut all = Vec::new();
        for (i, a) in adrs.iter().enumerate() { prog(i, adrs.len()); all.extend(Self::extract_from_adr(a)?); }
        Ok(all)
    }
}
