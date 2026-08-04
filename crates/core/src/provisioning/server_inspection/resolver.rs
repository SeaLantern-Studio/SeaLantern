use super::model::{Attributed, Detected, DetectionCandidate, EvidenceId};

#[derive(Debug, Clone)]
pub(crate) struct DetectionClaim<T> {
    pub(crate) value: T,
    pub(crate) evidence: EvidenceId,
    pub(crate) weight: u8,
    pub(crate) correlation_group: String,
}

struct CandidateState<T> {
    value: T,
    maximum_weight: u8,
    evidence: Vec<EvidenceId>,
    correlation_groups: Vec<String>,
    insertion_index: usize,
}

/// 同一候选的独立来源只提供有限加分，避免同一文件中的重复字段淹没冲突证据。
pub(crate) fn resolve<T>(claims: Vec<DetectionClaim<T>>) -> Detected<T>
where
    T: Eq,
{
    resolve_with_minimum_confidence(claims, 0)
}

pub(crate) fn resolve_with_minimum_confidence<T>(
    claims: Vec<DetectionClaim<T>>,
    minimum_confidence: u8,
) -> Detected<T>
where
    T: Eq,
{
    let candidates = rank_candidates(claims);
    let Some(winner) = candidates.first() else {
        return Detected::default();
    };
    let conflicts = candidates.get(1).is_some_and(|runner_up| {
        (winner.confidence >= 80 && runner_up.confidence >= 80)
            || (runner_up.confidence >= 50
                && winner.confidence.saturating_sub(runner_up.confidence) < 15)
    });

    if conflicts {
        return Detected {
            value: None,
            confidence: winner.confidence,
            evidence: winner.evidence.clone(),
            alternatives: candidates,
        };
    }

    if winner.confidence < minimum_confidence.min(100) {
        return Detected {
            value: None,
            confidence: winner.confidence,
            evidence: winner.evidence.clone(),
            alternatives: candidates,
        };
    }

    let mut candidates = candidates.into_iter();
    let Some(winner) = candidates.next() else {
        return Detected::default();
    };
    Detected {
        value: Some(winner.value),
        confidence: winner.confidence,
        evidence: winner.evidence,
        alternatives: candidates.collect(),
    }
}

pub(crate) fn resolve_attributed<T>(claims: Vec<DetectionClaim<T>>) -> Vec<Attributed<T>>
where
    T: Eq,
{
    rank_candidates(claims)
        .into_iter()
        .map(|candidate| Attributed {
            value: candidate.value,
            confidence: candidate.confidence,
            evidence: candidate.evidence,
        })
        .collect()
}

fn rank_candidates<T>(claims: Vec<DetectionClaim<T>>) -> Vec<DetectionCandidate<T>>
where
    T: Eq,
{
    let mut candidates: Vec<CandidateState<T>> = Vec::new();
    for claim in claims {
        if let Some(candidate) = candidates
            .iter_mut()
            .find(|candidate| candidate.value == claim.value)
        {
            candidate.maximum_weight = candidate.maximum_weight.max(claim.weight.min(100));
            if !candidate.evidence.contains(&claim.evidence) {
                candidate.evidence.push(claim.evidence);
            }
            if !candidate
                .correlation_groups
                .contains(&claim.correlation_group)
            {
                candidate.correlation_groups.push(claim.correlation_group);
            }
            continue;
        }

        candidates.push(CandidateState {
            value: claim.value,
            maximum_weight: claim.weight.min(100),
            evidence: vec![claim.evidence],
            correlation_groups: vec![claim.correlation_group],
            insertion_index: candidates.len(),
        });
    }

    let mut candidates = candidates
        .into_iter()
        .map(|candidate| {
            let independent_boost = candidate
                .correlation_groups
                .len()
                .saturating_sub(1)
                .saturating_mul(5)
                .min(10) as u8;
            let confidence = candidate
                .maximum_weight
                .saturating_add(independent_boost)
                .min(100);
            (
                DetectionCandidate {
                    value: candidate.value,
                    confidence,
                    evidence: candidate.evidence,
                },
                candidate.insertion_index,
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|(left, left_index), (right, right_index)| {
        right
            .confidence
            .cmp(&left.confidence)
            .then_with(|| left_index.cmp(right_index))
    });

    candidates
        .into_iter()
        .map(|(candidate, _)| candidate)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{resolve, resolve_with_minimum_confidence, DetectionClaim};
    use crate::provisioning::server_inspection::EvidenceId;

    fn claim(value: &str, id: u32, weight: u8, group: &str) -> DetectionClaim<String> {
        DetectionClaim {
            value: value.to_string(),
            evidence: EvidenceId(id),
            weight,
            correlation_group: group.to_string(),
        }
    }

    #[test]
    fn independent_evidence_adds_a_bounded_boost() {
        let detected = resolve(vec![
            claim("paper", 0, 80, "manifest"),
            claim("paper", 1, 75, "maven-coordinate"),
            claim("paper", 2, 70, "jar-entry"),
            claim("paper", 3, 65, "class-entry"),
            claim("purpur", 4, 60, "filename"),
        ]);

        assert_eq!(detected.value.as_deref(), Some("paper"));
        assert_eq!(detected.confidence, 90);
        assert_eq!(detected.evidence.len(), 4);
    }

    #[test]
    fn strong_conflicting_candidates_remain_unresolved() {
        let detected = resolve(vec![
            claim("paper", 0, 95, "coordinate"),
            claim("purpur", 1, 90, "patch-list"),
        ]);

        assert_eq!(detected.value, None);
        assert_eq!(detected.confidence, 95);
        assert_eq!(detected.alternatives.len(), 2);
    }

    #[test]
    fn low_confidence_candidate_remains_available_without_being_selected() {
        let detected =
            resolve_with_minimum_confidence(vec![claim("paper", 0, 25, "file-name")], 50);

        assert_eq!(detected.value, None);
        assert_eq!(detected.confidence, 25);
        assert_eq!(detected.evidence, vec![EvidenceId(0)]);
        assert_eq!(detected.alternatives.len(), 1);
        assert_eq!(detected.alternatives[0].value, "paper");
    }
}
