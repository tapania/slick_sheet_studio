//! Visual verification logic for AI-generated changes
//!
//! This module provides verification infrastructure for the planned
//! visual verification feature. Some items are not yet used internally.

#![allow(dead_code)]

/// Result of verifying a change
#[derive(Debug, Clone)]
pub enum VerificationResult {
    /// The change was successful and matches intent
    Success {
        /// Confidence level (0.0 - 1.0)
        confidence: f64,
        /// Description of what was verified
        message: String,
    },
    /// The change needs to be retried
    NeedsRetry {
        /// Why it needs to be retried
        reason: String,
        /// Suggestion for improvement
        suggestion: String,
    },
    /// The change failed and cannot be retried
    Failed {
        /// Error description
        error: String,
    },
}

impl VerificationResult {
    /// Check if the verification was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if the change needs to be retried
    pub fn needs_retry(&self) -> bool {
        matches!(self, Self::NeedsRetry { .. })
    }

    /// Get the suggestion if this result needs retry
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::NeedsRetry { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }
}

/// Verify a change by checking the output
///
/// This is a simplified verification that doesn't use vision LLM.
/// For full visual verification, use `verify_change_with_vision`.
pub fn verify_change(
    _request: &str,
    svg_output: Option<String>,
    compilation_error: Option<String>,
) -> VerificationResult {
    // If there's a compilation error, return failed
    if let Some(error) = compilation_error {
        return VerificationResult::Failed { error };
    }

    // Check SVG output
    let Some(svg) = svg_output else {
        return VerificationResult::NeedsRetry {
            reason: "No output generated".to_string(),
            suggestion: "Ensure the code produces visible content".to_string(),
        };
    };

    if svg.is_empty() {
        return VerificationResult::NeedsRetry {
            reason: "Empty SVG output".to_string(),
            suggestion: "Add visible content to the document".to_string(),
        };
    }

    // Basic checks on SVG content
    if svg.contains("<svg") && svg.len() > 100 {
        VerificationResult::Success {
            confidence: 0.8,
            message: "Output generated successfully".to_string(),
        }
    } else {
        VerificationResult::NeedsRetry {
            reason: "SVG output appears incomplete".to_string(),
            suggestion: "Check that all content is properly formatted".to_string(),
        }
    }
}

/// Parse a vision LLM response for verification
pub fn parse_verification_response(response: &str) -> VerificationResult {
    // Try to parse as JSON
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        let matches_intent = parsed
            .get("matches_intent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let confidence = parsed
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        let issues: Vec<String> = parsed
            .get("issues")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let suggestion = parsed
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if matches_intent && confidence >= 0.7 {
            VerificationResult::Success {
                confidence,
                message: if issues.is_empty() {
                    "Change verified successfully".to_string()
                } else {
                    format!("Verified with minor issues: {}", issues.join(", "))
                },
            }
        } else {
            VerificationResult::NeedsRetry {
                reason: if issues.is_empty() {
                    "Output does not match intent".to_string()
                } else {
                    issues.join("; ")
                },
                suggestion,
            }
        }
    } else {
        // Non-JSON response, try to interpret
        let lower = response.to_lowercase();
        if lower.contains("looks good") || lower.contains("matches") || lower.contains("correct") {
            VerificationResult::Success {
                confidence: 0.7,
                message: "Verified based on text response".to_string(),
            }
        } else {
            VerificationResult::NeedsRetry {
                reason: "Could not verify output".to_string(),
                suggestion: "Please review the output manually".to_string(),
            }
        }
    }
}
