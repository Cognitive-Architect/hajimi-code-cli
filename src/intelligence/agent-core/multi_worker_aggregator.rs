//! DEBT-LINES-B03B: Extracted multi-worker aggregation logic from reflector.rs::reflect_multi()
//! Computes success rates, determines critique severity, and aggregates worker results.

use crate::swarm::WorkerResult;
use crate::{Critique, CritiqueSeverity};

/// Aggregates multiple worker results into a unified critique.
pub struct MultiWorkerAggregator;

impl MultiWorkerAggregator {
    /// Aggregate worker results into a Critique and overall confidence score.
    pub fn aggregate_results(results: &[WorkerResult]) -> (Critique, f32) {
        let success_rate = Self::compute_success_rate(results);
        let severity = Self::determine_severity(success_rate);

        let issues: Vec<String> = results
            .iter()
            .filter(|r| !r.success)
            .map(|r| {
                format!(
                    "Worker {} failed: {}",
                    r.worker_id,
                    r.error.as_ref().unwrap_or(&r.output)
                )
            })
            .collect();

        let suggestions = if success_rate < 1.0 {
            vec![
                "Retry failed workers".to_string(),
                "Review resource allocation".to_string(),
            ]
        } else {
            vec!["Continue with next tasks".to_string()]
        };

        let critique = Critique {
            success: success_rate == 1.0,
            issues,
            suggestions,
            severity,
        };
        (critique, success_rate)
    }

    /// Compute the fraction of successful worker results.
    pub fn compute_success_rate(results: &[WorkerResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }
        let success_count = results.iter().filter(|r| r.success).count();
        success_count as f32 / results.len() as f32
    }

    /// Map success rate to a critique severity level.
    pub fn determine_severity(success_rate: f32) -> CritiqueSeverity {
        if success_rate == 0.0 {
            CritiqueSeverity::Critical
        } else if success_rate < 0.5 {
            CritiqueSeverity::High
        } else if success_rate < 1.0 {
            CritiqueSeverity::Medium
        } else {
            CritiqueSeverity::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::WorkerMetrics;
    use crate::swarm::WorkerResult;

    #[test]
    fn test_compute_success_rate_all_success() {
        let results = vec![
            WorkerResult::success("t1", "w1".to_string(), "ok1", WorkerMetrics::new(10)),
            WorkerResult::success("t2", "w2".to_string(), "ok2", WorkerMetrics::new(20)),
        ];
        assert_eq!(MultiWorkerAggregator::compute_success_rate(&results), 1.0);
        let (critique, confidence) = MultiWorkerAggregator::aggregate_results(&results);
        assert!(critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::Low);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_compute_success_rate_all_fail() {
        let results = vec![
            WorkerResult::failure("t1", "w1".to_string(), "e1", WorkerMetrics::new(10)),
            WorkerResult::failure("t2", "w2".to_string(), "e2", WorkerMetrics::new(20)),
        ];
        assert_eq!(MultiWorkerAggregator::compute_success_rate(&results), 0.0);
        let (critique, confidence) = MultiWorkerAggregator::aggregate_results(&results);
        assert!(!critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::Critical);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_compute_success_rate_partial() {
        let results = vec![
            WorkerResult::success("t1", "w1".to_string(), "ok", WorkerMetrics::new(10)),
            WorkerResult::failure("t2", "w2".to_string(), "e2", WorkerMetrics::new(20)),
            WorkerResult::failure("t3", "w3".to_string(), "e3", WorkerMetrics::new(30)),
        ];
        assert!((MultiWorkerAggregator::compute_success_rate(&results) - 1.0 / 3.0).abs() < 0.001);
        let (critique, confidence) = MultiWorkerAggregator::aggregate_results(&results);
        assert!(!critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::High);
        assert!((confidence - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_determine_severity_edge_cases() {
        assert_eq!(
            MultiWorkerAggregator::determine_severity(0.0),
            CritiqueSeverity::Critical
        );
        assert_eq!(
            MultiWorkerAggregator::determine_severity(0.49),
            CritiqueSeverity::High
        );
        assert_eq!(
            MultiWorkerAggregator::determine_severity(0.5),
            CritiqueSeverity::Medium
        );
        assert_eq!(
            MultiWorkerAggregator::determine_severity(0.99),
            CritiqueSeverity::Medium
        );
        assert_eq!(
            MultiWorkerAggregator::determine_severity(1.0),
            CritiqueSeverity::Low
        );
    }

    #[test]
    fn test_aggregate_empty_results() {
        let results: Vec<WorkerResult> = vec![];
        assert_eq!(MultiWorkerAggregator::compute_success_rate(&results), 0.0);
        let (critique, confidence) = MultiWorkerAggregator::aggregate_results(&results);
        assert!(!critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::Critical);
        assert_eq!(confidence, 0.0);
    }
}
