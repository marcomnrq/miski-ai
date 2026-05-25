use crate::models::{LabelledSegment, Segment};

/// Silence-gap-based speaker turn detector.
///
/// Detects speaker turns by looking for silence gaps in the transcript segments.
/// When a gap exceeds `min_silence_ms`, the speaker label toggles between
/// "Speaker A" and "Speaker B".
pub struct Diariser {
    /// Minimum silence gap (ms) to consider a speaker turn. Default: 700.
    pub min_silence_ms: i64,
    /// Minimum segment duration (ms) to include. Default: 300.
    pub min_segment_ms: i64,
}

impl Diariser {
    pub fn new() -> Self {
        Self {
            min_silence_ms: 700,
            min_segment_ms: 300,
        }
    }

    /// Detect speaker turns in a list of whisper segments.
    ///
    /// Returns labelled segments with alternating speaker labels:
    /// "Speaker A" and "Speaker B". Consecutive same-speaker segments
    /// are collapsed into a single segment.
    pub fn detect_turns(&self, segments: &[Segment]) -> Vec<LabelledSegment> {
        if segments.is_empty() {
            return Vec::new();
        }

        // 1. Filter out segments shorter than min_segment_ms
        let filtered: Vec<&Segment> = segments
            .iter()
            .filter(|s| (s.end_ms - s.start_ms) >= self.min_segment_ms)
            .collect();

        // Use filtered if available, otherwise use all segments
        let working: Vec<&Segment> = if filtered.is_empty() {
            segments.iter().collect()
        } else {
            filtered
        };

        // 2. Label segments based on silence gaps
        let speakers = ["Speaker A", "Speaker B"];
        let mut speaker_idx: usize = 0;
        let mut labelled = Vec::new();

        for (i, seg) in working.iter().enumerate() {
            if i > 0 {
                let prev = working[i - 1];
                let gap = seg.start_ms - prev.end_ms;
                if gap > self.min_silence_ms {
                    // Toggle speaker on silence gap
                    speaker_idx = (speaker_idx + 1) % 2;
                }
            }

            labelled.push(LabelledSegment {
                speaker: speakers[speaker_idx].to_string(),
                start_ms: seg.start_ms,
                end_ms: seg.end_ms,
                text: seg.text.clone(),
            });
        }

        // 3. Collapse consecutive same-speaker segments
        self.collapse_same_speaker(labelled)
    }

    /// Merge consecutive segments from the same speaker into one.
    fn collapse_same_speaker(&self, segments: Vec<LabelledSegment>) -> Vec<LabelledSegment> {
        if segments.is_empty() {
            return segments;
        }

        let mut result = Vec::new();
        let mut current = segments[0].clone();

        for seg in segments.iter().skip(1) {
            if seg.speaker == current.speaker {
                // Merge: extend end time and append text
                current.end_ms = seg.end_ms;
                current.text = format!("{} {}", current.text.trim(), seg.text.trim());
            } else {
                result.push(current);
                current = seg.clone();
            }
        }
        result.push(current);

        result
    }
}