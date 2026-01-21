//! @ai:module:intent Score quality of inferred AICMS annotations
//! @ai:module:layer application
//! @ai:module:public_api AnnotationScorer, AnnotationScore
//! @ai:module:stateless true

use regex::Regex;

/// @ai:intent Score of annotation inference quality
#[derive(Debug, Clone)]
pub struct AnnotationScore {
    pub completeness: f64,
    pub accuracy: f64,
    pub overall: f64,
    pub details: ScoringDetails,
}

/// @ai:intent Detailed breakdown of scoring
#[derive(Debug, Clone)]
pub struct ScoringDetails {
    pub has_intent: bool,
    pub has_pre: bool,
    pub has_post: bool,
    pub has_effects: bool,
    pub has_example: bool,
    pub intent_quality: f64,
    pub example_count: u32,
    pub matched_expected: u32,
    pub total_expected: u32,
}

/// @ai:intent Trait for annotation scoring
pub trait AnnotationScorerTrait: Send + Sync {
    /// @ai:intent Score inferred annotations
    fn score(&self, code: &str, expected_annotations: &[String]) -> AnnotationScore;
}

/// @ai:intent Scores the quality of inferred AICMS annotations
pub struct AnnotationScorer {
    intent_regex: Regex,
    pre_regex: Regex,
    post_regex: Regex,
    effects_regex: Regex,
    example_regex: Regex,
}

impl AnnotationScorer {
    /// @ai:intent Create a new annotation scorer
    /// @ai:effects pure
    pub fn new() -> Self {
        Self {
            intent_regex: Regex::new(r"@ai:intent\s+(.+)").unwrap(),
            pre_regex: Regex::new(r"@ai:pre\s+(.+)").unwrap(),
            post_regex: Regex::new(r"@ai:post\s+(.+)").unwrap(),
            effects_regex: Regex::new(r"@ai:effects\s+(.+)").unwrap(),
            example_regex: Regex::new(r"@ai:example\s+\(([^)]+)\)\s*->\s*(.+)").unwrap(),
        }
    }

    /// @ai:intent Check if code has a valid intent annotation
    /// @ai:effects pure
    fn check_intent(&self, code: &str) -> (bool, f64) {
        if let Some(cap) = self.intent_regex.captures(code) {
            let intent = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let quality = score_intent_quality(intent);
            (true, quality)
        } else {
            (false, 0.0)
        }
    }

    /// @ai:intent Count examples in code
    /// @ai:effects pure
    fn count_examples(&self, code: &str) -> u32 {
        self.example_regex.captures_iter(code).count() as u32
    }

    /// @ai:intent Count matched expected annotations
    /// @ai:effects pure
    fn count_matched(&self, code: &str, expected: &[String]) -> u32 {
        let code_lower = code.to_lowercase();

        expected
            .iter()
            .filter(|ann| code_lower.contains(&ann.to_lowercase()))
            .count() as u32
    }
}

impl Default for AnnotationScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// @ai:intent Score intent quality based on heuristics
/// @ai:effects pure
fn score_intent_quality(intent: &str) -> f64 {
    let mut score: f64 = 0.0;
    let words: Vec<&str> = intent.split_whitespace().collect();

    if words.len() >= 3 {
        score += 0.3;
    }

    if words.len() >= 5 {
        score += 0.2;
    }

    let action_verbs = [
        "calculate",
        "compute",
        "return",
        "validate",
        "check",
        "process",
        "convert",
        "transform",
        "find",
        "search",
        "create",
        "build",
        "parse",
        "format",
        "handle",
        "execute",
        "perform",
    ];

    let first_word = words.first().map(|w| w.to_lowercase()).unwrap_or_default();

    if action_verbs.contains(&first_word.as_str()) {
        score += 0.3;
    }

    if !intent.ends_with('.') {
        score += 0.1;
    }

    if intent.len() > 10 && intent.len() < 100 {
        score += 0.1;
    }

    score.min(1.0)
}

impl AnnotationScorerTrait for AnnotationScorer {
    /// @ai:intent Score inferred annotations against expectations
    /// @ai:effects pure
    fn score(&self, code: &str, expected_annotations: &[String]) -> AnnotationScore {
        let (has_intent, intent_quality) = self.check_intent(code);
        let has_pre = self.pre_regex.is_match(code);
        let has_post = self.post_regex.is_match(code);
        let has_effects = self.effects_regex.is_match(code);
        let has_example = self.example_regex.is_match(code);
        let example_count = self.count_examples(code);

        let completeness = calculate_completeness(has_intent, has_pre, has_post, has_effects, has_example);

        let matched = self.count_matched(code, expected_annotations);
        let total = expected_annotations.len() as u32;

        let accuracy = if total == 0 {
            1.0
        } else {
            matched as f64 / total as f64
        };

        let overall = (completeness * 0.4) + (accuracy * 0.4) + (intent_quality * 0.2);

        AnnotationScore {
            completeness,
            accuracy,
            overall,
            details: ScoringDetails {
                has_intent,
                has_pre,
                has_post,
                has_effects,
                has_example,
                intent_quality,
                example_count,
                matched_expected: matched,
                total_expected: total,
            },
        }
    }
}

/// @ai:intent Calculate completeness score
/// @ai:effects pure
fn calculate_completeness(
    has_intent: bool,
    has_pre: bool,
    has_post: bool,
    has_effects: bool,
    has_example: bool,
) -> f64 {
    let mut score = 0.0;

    if has_intent {
        score += 0.4;
    }

    if has_pre {
        score += 0.15;
    }

    if has_post {
        score += 0.15;
    }

    if has_effects {
        score += 0.15;
    }

    if has_example {
        score += 0.15;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_complete_annotations() {
        let scorer = AnnotationScorer::new();
        let code = r#"
/// @ai:intent Calculate the factorial of a number
/// @ai:pre n >= 0
/// @ai:post result >= 1
/// @ai:effects pure
/// @ai:example (5) -> 120
fn factorial(n: u64) -> u64 { 1 }
"#;

        let score = scorer.score(code, &[]);
        assert!(score.completeness > 0.9);
        assert!(score.details.has_intent);
        assert!(score.details.has_example);
    }

    #[test]
    fn test_score_minimal_annotations() {
        let scorer = AnnotationScorer::new();
        let code = "fn factorial(n: u64) -> u64 { 1 }";

        let score = scorer.score(code, &[]);
        assert!(score.completeness < 0.1);
    }

    #[test]
    fn test_score_intent_quality() {
        let good = score_intent_quality("Calculate the factorial of a given number");
        let bad = score_intent_quality("do");
        assert!(good > bad);
    }
}
