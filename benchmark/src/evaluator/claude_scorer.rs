//! @ai:module:intent Claude-based scoring of implementations
//! @ai:module:layer application
//! @ai:module:public_api ClaudeScorer, ComparisonScore, ImplementationScore
//! @ai:module:stateless true

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// @ai:intent Score for a single implementation aspect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectScore {
    pub score: u8,
    pub reason: String,
}

/// @ai:intent Detailed score breakdown for an implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationScore {
    /// Overall score 0-100
    pub overall: u8,
    /// Did the implementation match the specified intent?
    pub intent_match: AspectScore,
    /// How well were edge cases handled?
    pub edge_cases: AspectScore,
    /// Code quality and readability
    pub code_quality: AspectScore,
    /// How well were AICMS annotations used/followed?
    pub annotation_compliance: AspectScore,
}

/// @ai:intent Comparison result between baseline and AICMS implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonScore {
    pub baseline: ImplementationScore,
    pub aicms: ImplementationScore,
    pub winner: String,
    pub summary: String,
}

/// @ai:intent Trait for scoring implementations
pub trait ClaudeScorerTrait: Send + Sync {
    /// @ai:intent Compare and score two implementations by their directories
    fn compare_dirs(
        &self,
        task_spec: &str,
        baseline_dir: &Path,
        aicms_dir: &Path,
    ) -> Result<ComparisonScore>;
}

/// @ai:intent Uses Claude Code CLI to score implementations
pub struct ClaudeScorer {
    prompt_template: String,
}

impl ClaudeScorer {
    /// @ai:intent Create a new Claude scorer with a prompt template
    /// @ai:effects pure
    pub fn new(prompt_template: String) -> Self {
        Self { prompt_template }
    }

    /// @ai:intent Build the comparison prompt by substituting directory paths
    /// @ai:effects pure
    fn build_prompt(&self, task_spec: &str, baseline_dir: &Path, aicms_dir: &Path) -> String {
        self.prompt_template
            .replace("{{TASK_SPEC}}", task_spec)
            .replace("{{BASELINE_DIR}}", &baseline_dir.display().to_string())
            .replace("{{AICMS_DIR}}", &aicms_dir.display().to_string())
    }

    /// @ai:intent Parse Claude's JSON response
    /// @ai:effects pure
    fn parse_response(response: &str) -> Result<ComparisonScore> {
        // Try to extract JSON from the response
        let json_str = extract_json(response)?;
        let score: ComparisonScore = serde_json::from_str(&json_str)?;
        Ok(score)
    }
}

impl Default for ClaudeScorer {
    fn default() -> Self {
        Self::new(default_comparison_prompt())
    }
}

/// @ai:intent Default comparison prompt template
/// @ai:effects pure
pub fn default_comparison_prompt() -> String {
    r#"You are evaluating two implementations of the same task. Read and compare the source files.

## Task Specification
{{TASK_SPEC}}

## Directories to Compare
- **Baseline** (no AICMS context): {{BASELINE_DIR}}
- **AICMS** (with annotation context): {{AICMS_DIR}}

## IMPORTANT: Fair Comparison Rules

**IGNORE all `@ai:*` annotation comments when scoring.** Do not give any advantage or disadvantage to code based on the presence or absence of `@ai:intent`, `@ai:pre`, `@ai:post`, or any other AICMS annotations.

Focus ONLY on:
- The actual code implementation
- The logic and algorithms used
- Error handling and edge cases
- Code structure and readability

## Instructions
1. Read all source files in both directories (ignore _claude_interaction.log and target/)
2. **Strip out all `@ai:*` annotations mentally** before evaluating
3. Compare the implementations on the criteria below
4. Output ONLY the JSON result (no markdown, no explanation)

## Scoring Criteria (0-100 for each)
1. **Intent Match**: Does the actual implementation correctly fulfill the specified task? (Ignore @ai:intent comments)
2. **Edge Cases**: Are edge cases handled correctly in the code? (Ignore @ai:pre/@ai:post comments)
3. **Code Quality**: Is the code clean, readable, and well-structured?
4. **Error Handling**: Does the code properly handle errors and invalid inputs?

## Required Output Format
Respond ONLY with valid JSON in this exact format:
{
  "baseline": {
    "overall": <0-100>,
    "intent_match": {"score": <0-100>, "reason": "<brief reason>"},
    "edge_cases": {"score": <0-100>, "reason": "<brief reason>"},
    "code_quality": {"score": <0-100>, "reason": "<brief reason>"},
    "annotation_compliance": {"score": <0-100>, "reason": "<brief reason for error handling>"}
  },
  "aicms": {
    "overall": <0-100>,
    "intent_match": {"score": <0-100>, "reason": "<brief reason>"},
    "edge_cases": {"score": <0-100>, "reason": "<brief reason>"},
    "code_quality": {"score": <0-100>, "reason": "<brief reason>"},
    "annotation_compliance": {"score": <0-100>, "reason": "<brief reason for error handling>"}
  },
  "winner": "<baseline|aicms|tie>",
  "summary": "<one sentence comparing the two implementations>"
}"#
    .to_string()
}

impl ClaudeScorerTrait for ClaudeScorer {
    /// @ai:intent Compare and score two implementations using Claude agentic mode
    /// @ai:effects io, network, fs:read
    fn compare_dirs(
        &self,
        task_spec: &str,
        baseline_dir: &Path,
        aicms_dir: &Path,
    ) -> Result<ComparisonScore> {
        use std::io::Write;
        use std::process::Stdio;

        let prompt = self.build_prompt(task_spec, baseline_dir, aicms_dir);

        // Run Claude in agentic mode to let it read files from directories
        let mut child = Command::new("claude")
            .arg("--print")
            .arg("--verbose")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Claude CLI stderr: {}", stderr);
        }

        let response = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("Claude comparison response: {}", response);

        Self::parse_response(&response)
    }
}

/// @ai:intent Extract JSON object from response that may contain extra text
/// @ai:effects pure
fn extract_json(response: &str) -> Result<String> {
    // Find the first { and last }
    let start = response
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("No JSON object found in response"))?;
    let end = response
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("No JSON object end found in response"))?;

    if end <= start {
        anyhow::bail!("Invalid JSON structure in response");
    }

    Ok(response[start..=end].to_string())
}

/// @ai:intent Mock scorer for testing
pub struct MockClaudeScorer {
    score: ComparisonScore,
}

impl MockClaudeScorer {
    /// @ai:intent Create a mock scorer with predetermined score
    pub fn new(score: ComparisonScore) -> Self {
        Self { score }
    }

    /// @ai:intent Create a mock scorer with default scores
    pub fn with_defaults() -> Self {
        Self {
            score: ComparisonScore {
                baseline: ImplementationScore {
                    overall: 70,
                    intent_match: AspectScore {
                        score: 70,
                        reason: "Mock baseline".to_string(),
                    },
                    edge_cases: AspectScore {
                        score: 60,
                        reason: "Mock baseline".to_string(),
                    },
                    code_quality: AspectScore {
                        score: 75,
                        reason: "Mock baseline".to_string(),
                    },
                    annotation_compliance: AspectScore {
                        score: 50,
                        reason: "Mock baseline".to_string(),
                    },
                },
                aicms: ImplementationScore {
                    overall: 85,
                    intent_match: AspectScore {
                        score: 90,
                        reason: "Mock AICMS".to_string(),
                    },
                    edge_cases: AspectScore {
                        score: 85,
                        reason: "Mock AICMS".to_string(),
                    },
                    code_quality: AspectScore {
                        score: 80,
                        reason: "Mock AICMS".to_string(),
                    },
                    annotation_compliance: AspectScore {
                        score: 90,
                        reason: "Mock AICMS".to_string(),
                    },
                },
                winner: "aicms".to_string(),
                summary: "Mock comparison".to_string(),
            },
        }
    }
}

impl ClaudeScorerTrait for MockClaudeScorer {
    fn compare_dirs(
        &self,
        _task_spec: &str,
        _baseline_dir: &Path,
        _aicms_dir: &Path,
    ) -> Result<ComparisonScore> {
        Ok(self.score.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_simple() {
        let response = r#"{"score": 85}"#;
        let result = extract_json(response).unwrap();
        assert_eq!(result, r#"{"score": 85}"#);
    }

    #[test]
    fn test_extract_json_with_prefix() {
        let response = r#"Here's the score: {"score": 85} end"#;
        let result = extract_json(response).unwrap();
        assert_eq!(result, r#"{"score": 85}"#);
    }

    #[test]
    fn test_build_prompt_contains_paths() {
        let scorer = ClaudeScorer::default();
        let baseline = Path::new("/tmp/baseline");
        let aicms = Path::new("/tmp/aicms");
        let prompt = scorer.build_prompt("spec", baseline, aicms);
        assert!(prompt.contains("/tmp/baseline"));
        assert!(prompt.contains("/tmp/aicms"));
        assert!(prompt.contains("spec"));
    }

    #[test]
    fn test_mock_scorer() {
        let scorer = MockClaudeScorer::with_defaults();
        let baseline = Path::new("/tmp/baseline");
        let aicms = Path::new("/tmp/aicms");
        let result = scorer.compare_dirs("spec", baseline, aicms).unwrap();
        assert_eq!(result.winner, "aicms");
        assert!(result.aicms.overall > result.baseline.overall);
    }
}
