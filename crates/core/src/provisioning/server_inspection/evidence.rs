use super::model::{
    DetectionEvidence, DetectionTarget, EvidenceId, EvidenceLocation, EvidenceSource,
};

#[derive(Debug, Default)]
pub(crate) struct EvidenceCollector {
    entries: Vec<DetectionEvidence>,
}

impl EvidenceCollector {
    pub(crate) fn push(&mut self, evidence: NewEvidence) -> EvidenceId {
        let id = EvidenceId(self.entries.len() as u32);
        self.entries.push(DetectionEvidence {
            id,
            detector: evidence.detector.to_string(),
            source: evidence.source,
            location: evidence.location,
            target: evidence.target,
            candidate: evidence.candidate,
            weight: evidence.weight.min(100),
            correlation_group: evidence.correlation_group.to_string(),
        });
        id
    }

    pub(crate) fn into_entries(self) -> Vec<DetectionEvidence> {
        self.entries
    }
}

pub(crate) struct NewEvidence {
    pub(crate) detector: &'static str,
    pub(crate) source: EvidenceSource,
    pub(crate) location: EvidenceLocation,
    pub(crate) target: DetectionTarget,
    pub(crate) candidate: String,
    pub(crate) weight: u8,
    pub(crate) correlation_group: &'static str,
}
