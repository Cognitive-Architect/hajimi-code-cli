//! Scoring rules for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::types::{SkillManifest, SkillMatchScore};

/// Computes the match score for a single Skill against the user input.
///
/// Scoring breakdown:
/// - trigger full match: triggers complete word / substring matched is +0.60
/// - name/title match: if input contains name or title (case-insensitive): +0.25
/// - description keyword match: if input contains keywords (case-insensitive) from description: +0.15
///
/// Max score capped at 1.0.
pub fn score_skill(input: &str, manifest: &SkillManifest) -> SkillMatchScore {
    let input_lower = input.to_lowercase();
    let mut score = 0.0;
    let mut matched_triggers = Vec::new();
    let mut matched_name_or_title = None;
    let mut matched_description_terms = Vec::new();

    // 1. Trigger matching
    for trigger in &manifest.triggers {
        let trigger_lower = trigger.to_lowercase();
        if input_lower.contains(&trigger_lower) {
            matched_triggers.push(trigger.clone());
        }
    }
    if !matched_triggers.is_empty() {
        score += 0.60;
    }

    // 2. Name/Title matching
    let name_lower = manifest.name.to_lowercase();
    let title_lower = manifest.title.to_lowercase();
    if input_lower.contains(&name_lower) {
        matched_name_or_title = Some(manifest.name.clone());
        score += 0.25;
    } else if input_lower.contains(&title_lower) {
        matched_name_or_title = Some(manifest.title.clone());
        score += 0.25;
    }

    // 3. Description keyword matching
    let phrases: Vec<&str> = manifest
        .description
        .split(|c: char| {
            c.is_ascii_punctuation()
                || c == '，'
                || c == '。'
                || c == '、'
                || c == '；'
                || c == '：'
                || c == '！'
                || c == '？'
                || c == '（'
                || c == '）'
                || c.is_whitespace()
        })
        .filter(|s| s.chars().count() >= 2)
        .collect();

    for phrase in phrases {
        let phrase_lower = phrase.to_lowercase();
        if input_lower.contains(&phrase_lower) {
            matched_description_terms.push(phrase.to_string());
        } else {
            let chars: Vec<char> = phrase_lower.chars().collect();
            for window in chars.windows(2) {
                let sub: String = window.iter().collect();
                if input_lower.contains(&sub) && !matched_description_terms.contains(&sub) {
                    matched_description_terms.push(sub);
                }
            }
        }
    }

    if !matched_description_terms.is_empty() {
        score += 0.15;
    }

    if score > 1.0 {
        score = 1.0;
    }

    // Construct a readable, clear reason explanation
    let mut reason_parts = Vec::new();
    if !matched_triggers.is_empty() {
        reason_parts.push(format!(
            "triggers matched: [{}] (+0.60)",
            matched_triggers.join(", ")
        ));
    }
    if let Some(ref name_or_title) = matched_name_or_title {
        reason_parts.push(format!("name/title matched: '{}' (+0.25)", name_or_title));
    }
    if !matched_description_terms.is_empty() {
        reason_parts.push(format!(
            "description keywords matched: [{}] (+0.15)",
            matched_description_terms.join(", ")
        ));
    }

    let reason = if reason_parts.is_empty() {
        "no matching triggers, name, title, or description keywords found".to_string()
    } else {
        reason_parts.join("; ")
    };

    SkillMatchScore {
        score,
        matched_triggers,
        matched_name_or_title,
        matched_description_terms,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{SkillPermissions, SkillRiskLevel};

    fn mock_manifest() -> SkillManifest {
        SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: "auto-save".to_string(),
            title: "自动存档".to_string(),
            description: "当任务推进、状态变化时，生成存档块。".to_string(),
            enabled: true,
            category: Some("handoff".to_string()),
            exclusive_group: Some("handoff-output".to_string()),
            triggers: vec![
                "自动存档".to_string(),
                "存档".to_string(),
                "项目状态".to_string(),
                "下一步".to_string(),
                "handoff".to_string(),
            ],
            risk_level: SkillRiskLevel::Low,
            entry: "SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        }
    }

    #[test]
    fn test_score_trigger_and_description() {
        let manifest = mock_manifest();
        let input = "现在任务推进得差不多了，帮我存档一下，记录下一步";
        let res = score_skill(input, &manifest);
        assert!(res.score >= 0.75); // 0.60 (triggers: "存档", "下一步") + 0.15 (description: "任务推进")
        assert!(res.matched_triggers.contains(&"存档".to_string()));
        assert!(res.matched_triggers.contains(&"下一步".to_string()));
        assert!(res
            .matched_description_terms
            .iter()
            .any(|t| t.contains("推进") || t.contains("任务")));
    }

    #[test]
    fn test_score_name_title() {
        let manifest = mock_manifest();
        let input = "进行自动存档吧";
        let res = score_skill(input, &manifest);
        assert_eq!(res.score, 1.0); // 0.60 (trigger: "自动存档") + 0.25 (title: "自动存档") + 0.15 (description: "存档")
        assert_eq!(res.matched_name_or_title, Some("自动存档".to_string()));
    }

    #[test]
    fn test_no_matches() {
        let manifest = mock_manifest();
        let input = "你好，写段 Rust 代码吧";
        let res = score_skill(input, &manifest);
        assert_eq!(res.score, 0.0);
        assert!(res.matched_triggers.is_empty());
        assert_eq!(res.matched_name_or_title, None);
        assert!(res.matched_description_terms.is_empty());
        assert!(res.reason.contains("no matching"));
    }
}
