//! Completion engine for providing code completions
//!
//! This module defines the core types and traits for the completion system.
//! It supports both static pattern-based completions and LLM-powered completions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Position in a text document (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column/character position (0-indexed)
    pub column: usize,
}

/// Request for code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The file path being edited
    pub file_path: String,
    /// The programming language (e.g., "rust", "python", "javascript")
    pub language: String,
    /// Current line content
    pub current_line: String,
    /// Cursor position in the document
    pub cursor_position: Position,
    /// Optional: surrounding context (lines before/after)
    #[serde(default)]
    pub context_before: Vec<String>,
    #[serde(default)]
    pub context_after: Vec<String>,
}

/// A single completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    /// The completion text to insert
    pub text: String,
    /// How many characters to move the cursor after insertion
    /// (useful for placing cursor inside parentheses, etc.)
    pub cursor_offset: i32,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Source of the completion ("static", "llm", "mcp", etc.)
    pub source: CompletionSource,
    /// Optional metadata about how this completion was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Source of a completion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionSource {
    /// Static pattern-based completion
    Static,
    /// LLM-generated completion
    Llm,
    /// MCP-assisted completion
    Mcp,
    /// RAG-enhanced completion
    Rag,
}

impl fmt::Display for CompletionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompletionSource::Static => write!(f, "static"),
            CompletionSource::Llm => write!(f, "llm"),
            CompletionSource::Mcp => write!(f, "mcp"),
            CompletionSource::Rag => write!(f, "rag"),
        }
    }
}

/// Response containing completion suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// List of completion suggestions (can be empty)
    pub completions: Vec<Completion>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Pattern types that can be detected in code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// Function definition start (e.g., "fn foo" or "function myFunc")
    FunctionStart,
    /// If statement
    IfStatement,
    /// For loop
    ForLoop,
    /// While loop
    WhileLoop,
    /// Struct/class definition
    StructDef,
    /// Impl block (Rust-specific)
    ImplBlock,
    /// Match/switch statement
    MatchStatement,
    /// Unknown/no pattern detected
    Unknown,
}

/// Trait for detecting patterns in code
pub trait PatternDetector: Send + Sync {
    /// Detect the pattern in the given line
    fn detect_pattern(&self, line: &str, language: &str) -> Pattern;

    /// Get a completion template for the detected pattern
    fn get_template(&self, pattern: Pattern, language: &str) -> Option<String>;
}

/// Trait for completion providers (static, LLM, etc.)
#[async_trait::async_trait]
pub trait CompletionProvider: Send + Sync {
    /// Generate completions for the given request
    async fn complete(&self, request: &CompletionRequest) -> Result<Vec<Completion>>;

    /// Name of this provider
    fn name(&self) -> &str;

    /// Whether this provider is available/enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Main completion engine that orchestrates different providers
pub struct CompletionEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
}

impl CompletionEngine {
    /// Create a new completion engine
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a completion provider
    pub fn add_provider(&mut self, provider: Box<dyn CompletionProvider>) {
        self.providers.push(provider);
    }

    /// Get completions for the given request
    ///
    /// This will try all enabled providers and combine their results
    pub async fn get_completions(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let start = std::time::Instant::now();
        let mut all_completions = Vec::new();

        // Try each provider
        for provider in &self.providers {
            if !provider.is_enabled() {
                continue;
            }

            match provider.complete(request).await {
                Ok(completions) => {
                    tracing::debug!(
                        "Provider '{}' returned {} completions",
                        provider.name(),
                        completions.len()
                    );
                    all_completions.extend(completions);
                }
                Err(e) => {
                    tracing::warn!(
                        "Provider '{}' failed: {}",
                        provider.name(),
                        e
                    );
                }
            }
        }

        // Sort by confidence (highest first) and deduplicate
        all_completions.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove duplicates (keep highest confidence)
        all_completions.dedup_by(|a, b| a.text == b.text);

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(CompletionResponse {
            completions: all_completions,
            processing_time_ms,
        })
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_serialization() {
        let pos = Position { line: 10, column: 5 };
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#"{"line":10,"column":5}"#);
    }

    #[test]
    fn test_completion_source_display() {
        assert_eq!(CompletionSource::Static.to_string(), "static");
        assert_eq!(CompletionSource::Llm.to_string(), "llm");
    }
}
