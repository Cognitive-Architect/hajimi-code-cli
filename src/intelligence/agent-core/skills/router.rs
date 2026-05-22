//! SkillRouter — Score and route local skill packs against input.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::registry::SkillRegistry;
use crate::skills::scoring::score_skill;
use crate::skills::types::{SkillMatch, SkillRouteReceipt, SkillRouterConfig};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The output result of a SkillRouter operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SkillRouteResult {
    pub selected: Vec<SkillMatch>,
    pub rejected: Vec<SkillMatch>,
    pub receipt: SkillRouteReceipt,
}

/// Router that evaluates user inputs and schedules active skills.
pub struct SkillRouter {
    pub registry: Arc<SkillRegistry>,
    pub config: SkillRouterConfig,
}

impl SkillRouter {
    /// Creates a new SkillRouter.
    pub fn new(registry: Arc<SkillRegistry>, config: SkillRouterConfig) -> Self {
        Self { registry, config }
    }

    /// Evaluates user input against scanned skills and decides which skills to activate.
    pub fn route(&self, input: &str) -> SkillRouteResult {
        let mut all_matches = Vec::new();

        // 1. Score each enabled skill in the registry
        for manifest in self.registry.list() {
            if !manifest.enabled {
                continue;
            }

            let match_score = score_skill(input, manifest);
            all_matches.push((manifest, match_score));
        }

        let mut selected_candidates: Vec<SkillMatch> = Vec::new();
        let mut rejected: Vec<SkillMatch> = Vec::new();

        // Group matches by exclusive_group to detect conflicts
        let mut group_matches: HashMap<Option<String>, Vec<(usize, f32)>> = HashMap::new();
        for (idx, (manifest, match_score)) in all_matches.iter().enumerate() {
            let key = manifest.exclusive_group.clone();
            group_matches
                .entry(key)
                .or_default()
                .push((idx, match_score.score));
        }

        // For each group, determine who is the winner or if they all pass
        for (group_opt, mut indices) in group_matches {
            if let Some(ref group_name) = group_opt {
                // If there's an exclusive group, sort by score descending and keep only the highest
                indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                let (winner_idx, winner_score) = indices[0];
                let (winner_manifest, winner_match_score) = &all_matches[winner_idx];

                // Process the winner
                if winner_score >= self.config.threshold {
                    selected_candidates.push(SkillMatch {
                        name: winner_manifest.name.clone(),
                        score: winner_score,
                        reason: winner_match_score.reason.clone(),
                        matched_terms: winner_match_score.matched_description_terms.clone(),
                        risk_level: winner_manifest.risk_level,
                        category: winner_manifest.category.clone(),
                        exclusive_group: winner_manifest.exclusive_group.clone(),
                    });
                } else {
                    rejected.push(SkillMatch {
                        name: winner_manifest.name.clone(),
                        score: winner_score,
                        reason: format!(
                            "below threshold {} (score={:.2}): {}",
                            self.config.threshold, winner_score, winner_match_score.reason
                        ),
                        matched_terms: winner_match_score.matched_description_terms.clone(),
                        risk_level: winner_manifest.risk_level,
                        category: winner_manifest.category.clone(),
                        exclusive_group: winner_manifest.exclusive_group.clone(),
                    });
                }

                // Process the losers in the same exclusive group
                for &(loser_idx, loser_score) in &indices[1..] {
                    let (loser_manifest, loser_match_score) = &all_matches[loser_idx];
                    rejected.push(SkillMatch {
                        name: loser_manifest.name.clone(),
                        score: loser_score,
                        reason: format!(
                            "exclusive_group conflict: rejected in favor of {} (exclusive_group='{}', winner_score={:.2}, this_score={:.2})",
                            winner_manifest.name, group_name, winner_score, loser_score
                        ),
                        matched_terms: loser_match_score.matched_description_terms.clone(),
                        risk_level: loser_manifest.risk_level,
                        category: loser_manifest.category.clone(),
                        exclusive_group: loser_manifest.exclusive_group.clone(),
                    });
                }
            } else {
                // No exclusive group, each is processed independently
                for (idx, score) in indices {
                    let (manifest, match_score) = &all_matches[idx];
                    if score >= self.config.threshold {
                        selected_candidates.push(SkillMatch {
                            name: manifest.name.clone(),
                            score,
                            reason: match_score.reason.clone(),
                            matched_terms: match_score.matched_description_terms.clone(),
                            risk_level: manifest.risk_level,
                            category: manifest.category.clone(),
                            exclusive_group: None,
                        });
                    } else {
                        rejected.push(SkillMatch {
                            name: manifest.name.clone(),
                            score,
                            reason: format!(
                                "below threshold {} (score={:.2}): {}",
                                self.config.threshold, score, match_score.reason
                            ),
                            matched_terms: match_score.matched_description_terms.clone(),
                            risk_level: manifest.risk_level,
                            category: manifest.category.clone(),
                            exclusive_group: None,
                        });
                    }
                }
            }
        }

        // Sort candidates by score descending to apply Top-K limit
        selected_candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected = Vec::new();
        for (i, candidate) in selected_candidates.into_iter().enumerate() {
            if i < self.config.max_active_skills {
                selected.push(candidate);
            } else {
                let name = candidate.name.clone();
                let score = candidate.score;
                let matched_terms = candidate.matched_terms.clone();
                let risk_level = candidate.risk_level;
                let category = candidate.category.clone();
                let exclusive_group = candidate.exclusive_group.clone();
                rejected.push(SkillMatch {
                    name,
                    score,
                    reason: format!(
                        "rejected due to max_active_skills limit (top {} selected, score={:.2})",
                        self.config.max_active_skills, score
                    ),
                    matched_terms,
                    risk_level,
                    category,
                    exclusive_group,
                });
            }
        }

        // Sort rejected by name for clean, deterministic lists
        rejected.sort_by(|a, b| a.name.cmp(&b.name));

        // Create route receipt
        let input_hash = {
            let mut hasher = DefaultHasher::new();
            input.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

        let timestamp = chrono::Utc::now().to_rfc3339();

        let receipt = SkillRouteReceipt {
            input_hash,
            router_version: "hajimi.skill.router.v0".to_string(),
            selected: selected.clone(),
            rejected: rejected.clone(),
            timestamp,
        };

        SkillRouteResult {
            selected,
            rejected,
            receipt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{SkillManifest, SkillPermissions, SkillRiskLevel};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    fn write_mock_skill(
        dir: &std::path::Path,
        name: &str,
        title: &str,
        desc: &str,
        triggers: Vec<&str>,
        group: Option<&str>,
    ) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();

        let manifest = SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: name.to_string(),
            title: title.to_string(),
            description: desc.to_string(),
            enabled: true,
            category: Some("test".to_string()),
            exclusive_group: group.map(|s| s.to_string()),
            triggers: triggers.into_iter().map(|s| s.to_string()).collect(),
            risk_level: SkillRiskLevel::Low,
            entry: "SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let mut f = File::create(skill_dir.join("skill.json")).unwrap();
        f.write_all(json.as_bytes()).unwrap();

        let mut f2 = File::create(skill_dir.join("SKILL.md")).unwrap();
        f2.write_all(b"Mock content").unwrap();
    }

    #[test]
    fn test_exclusive_group_conflict_resolution() {
        let dir = tempdir().unwrap();
        // Two skills in same exclusive group "handoff"
        write_mock_skill(
            dir.path(),
            "auto-save",
            "自动存档",
            "当任务推进时，生成存档。",
            vec!["存档", "记录"],
            Some("handoff"),
        );
        write_mock_skill(
            dir.path(),
            "project-handoff",
            "项目移交",
            "移交当前项目。",
            vec!["移交", "记录"],
            Some("handoff"),
        );

        let registry = Arc::new(SkillRegistry::scan(dir.path()).unwrap());
        let config = SkillRouterConfig {
            threshold: 0.55,
            max_active_skills: 3,
        };
        let router = SkillRouter::new(registry, config);

        // Input triggers "存档" (auto-save gets 0.60) and "记录" (both get 0.60).
        // Let's check when auto-save wins because of higher keyword overlap: "当任务推进时，生成存档。" overlaps with "存档"
        let input = "帮我存档一下";
        let res = router.route(input);

        assert_eq!(res.selected.len(), 1);
        assert_eq!(res.selected[0].name, "auto-save");

        assert_eq!(res.rejected.len(), 1);
        assert_eq!(res.rejected[0].name, "project-handoff");
        assert!(res.rejected[0].reason.contains("exclusive_group conflict"));
    }

    #[test]
    fn test_top_k_limits() {
        let dir = tempdir().unwrap();
        // Four distinct skills
        write_mock_skill(dir.path(), "skill-a", "A", "A", vec!["A"], None);
        write_mock_skill(dir.path(), "skill-b", "B", "B", vec!["B"], None);
        write_mock_skill(dir.path(), "skill-c", "C", "C", vec!["C"], None);
        write_mock_skill(dir.path(), "skill-d", "D", "D", vec!["D"], None);

        let registry = Arc::new(SkillRegistry::scan(dir.path()).unwrap());
        // max 3 active skills
        let config = SkillRouterConfig {
            threshold: 0.55,
            max_active_skills: 3,
        };
        let router = SkillRouter::new(registry, config);

        // Input triggers all four
        let input = "A B C D";
        let res = router.route(input);

        assert_eq!(res.selected.len(), 3);
        assert_eq!(res.rejected.len(), 1);
        assert!(res.rejected[0].reason.contains("max_active_skills limit"));
    }

    #[derive(serde::Deserialize)]
    struct RouteCase {
        input: String,
        should_trigger: bool,
        matched_triggers: Vec<String>,
    }

    #[test]
    fn test_router_with_route_cases_fixture() {
        let base_dir = std::env::current_dir().unwrap();
        let mut fixture_dir = base_dir.join("tests/fixtures/skills");
        if !fixture_dir.exists() {
            fixture_dir = base_dir.join("../../../tests/fixtures/skills");
        }
        let auto_save_dir = fixture_dir.join("auto-save");
        let cases_path = auto_save_dir.join("evals/route_cases.json");

        let file_content =
            fs::read_to_string(&cases_path).expect("Failed to read route_cases.json");
        let cases: Vec<RouteCase> =
            serde_json::from_str(&file_content).expect("Failed to deserialize route_cases.json");

        let registry = Arc::new(SkillRegistry::scan(&fixture_dir).unwrap());
        let config = SkillRouterConfig {
            threshold: 0.55,
            max_active_skills: 3,
        };
        let router = SkillRouter::new(registry, config);

        for case in cases {
            let res = router.route(&case.input);
            let is_auto_save_selected = res.selected.iter().any(|m| m.name == "auto-save");

            if case.should_trigger {
                assert!(
                    is_auto_save_selected,
                    "Expected auto-save to be selected for input: '{}'",
                    case.input
                );
                if let Some(m) = res.selected.iter().find(|m| m.name == "auto-save") {
                    for expected_trigger in &case.matched_triggers {
                        assert!(
                            m.reason.contains(expected_trigger),
                            "Expected trigger '{}' to be mentioned in reason: '{}'",
                            expected_trigger,
                            m.reason
                        );
                    }
                }
            } else {
                assert!(
                    !is_auto_save_selected,
                    "Expected auto-save NOT to be selected for input: '{}'",
                    case.input
                );
            }
        }
    }
}
