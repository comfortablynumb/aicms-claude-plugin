You are evaluating two implementations of the same task. Read and compare the source files.

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
}
