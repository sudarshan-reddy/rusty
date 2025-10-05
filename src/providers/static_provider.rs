//! Static pattern-based completion provider
//!
//! This provider detects common code patterns and provides static completions

use anyhow::Result;
use regex::Regex;

use crate::completion::{
    Completion, CompletionProvider, CompletionRequest, CompletionSource, Pattern, PatternDetector,
};

/// Static pattern detector
pub struct StaticPatternDetector;

impl PatternDetector for StaticPatternDetector {
    fn detect_pattern(&self, line: &str, language: &str) -> Pattern {
        let trimmed = line.trim();

        match language {
            "rust" => self.detect_rust_pattern(trimmed),
            "python" => self.detect_python_pattern(trimmed),
            "javascript" | "typescript" => self.detect_js_pattern(trimmed),
            _ => Pattern::Unknown,
        }
    }

    fn get_template(&self, pattern: Pattern, language: &str) -> Option<String> {
        match (pattern, language) {
            // Rust templates
            (Pattern::FunctionStart, "rust") => Some("() {\n    \n}".to_string()),
            (Pattern::IfStatement, "rust") => Some(" {\n    \n}".to_string()),
            (Pattern::ForLoop, "rust") => Some(" {\n    \n}".to_string()),
            (Pattern::WhileLoop, "rust") => Some(" {\n    \n}".to_string()),
            (Pattern::StructDef, "rust") => Some(" {\n    \n}".to_string()),
            (Pattern::ImplBlock, "rust") => Some(" {\n    \n}".to_string()),
            (Pattern::MatchStatement, "rust") => Some(" {\n    \n}".to_string()),

            // Python templates
            (Pattern::FunctionStart, "python") => Some(":\n    ".to_string()),
            (Pattern::IfStatement, "python") => Some(":\n    ".to_string()),
            (Pattern::ForLoop, "python") => Some(":\n    ".to_string()),
            (Pattern::WhileLoop, "python") => Some(":\n    ".to_string()),
            (Pattern::StructDef, "python") => Some(":\n    ".to_string()),

            // JavaScript/TypeScript templates
            (Pattern::FunctionStart, "javascript" | "typescript") => {
                Some("() {\n    \n}".to_string())
            }
            (Pattern::IfStatement, "javascript" | "typescript") => Some(" {\n    \n}".to_string()),
            (Pattern::ForLoop, "javascript" | "typescript") => Some(" {\n    \n}".to_string()),
            (Pattern::WhileLoop, "javascript" | "typescript") => Some(" {\n    \n}".to_string()),

            _ => None,
        }
    }
}

impl StaticPatternDetector {
    fn detect_rust_pattern(&self, line: &str) -> Pattern {
        // Function definition
        if Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+")
            .unwrap()
            .is_match(line)
        {
            // Check if line doesn't already end with {
            if !line.contains('{') {
                return Pattern::FunctionStart;
            }
        }

        // If statement
        if Regex::new(r"^\s*if\s+").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::IfStatement;
        }

        // For loop
        if Regex::new(r"^\s*for\s+").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::ForLoop;
        }

        // While loop
        if Regex::new(r"^\s*while\s+").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::WhileLoop;
        }

        // Struct definition
        if Regex::new(r"^\s*(?:pub\s+)?struct\s+\w+")
            .unwrap()
            .is_match(line)
            && !line.contains('{')
        {
            return Pattern::StructDef;
        }

        // Impl block
        if Regex::new(r"^\s*impl(?:<[^>]+>)?\s+")
            .unwrap()
            .is_match(line)
            && !line.contains('{')
        {
            return Pattern::ImplBlock;
        }

        // Match statement
        if Regex::new(r"^\s*match\s+").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::MatchStatement;
        }

        Pattern::Unknown
    }

    fn detect_python_pattern(&self, line: &str) -> Pattern {
        // Function definition
        if Regex::new(r"^\s*(?:async\s+)?def\s+\w+")
            .unwrap()
            .is_match(line)
            && !line.ends_with(':')
        {
            return Pattern::FunctionStart;
        }

        // If statement
        if Regex::new(r"^\s*if\s+").unwrap().is_match(line) && !line.ends_with(':') {
            return Pattern::IfStatement;
        }

        // For loop
        if Regex::new(r"^\s*for\s+").unwrap().is_match(line) && !line.ends_with(':') {
            return Pattern::ForLoop;
        }

        // While loop
        if Regex::new(r"^\s*while\s+").unwrap().is_match(line) && !line.ends_with(':') {
            return Pattern::WhileLoop;
        }

        // Class definition
        if Regex::new(r"^\s*class\s+\w+").unwrap().is_match(line) && !line.ends_with(':') {
            return Pattern::StructDef;
        }

        Pattern::Unknown
    }

    fn detect_js_pattern(&self, line: &str) -> Pattern {
        // Function definition
        if Regex::new(
            r"^\s*(?:async\s+)?(?:function\s+\w+|const\s+\w+\s*=\s*(?:async\s+)?\([^)]*\)\s*=>)",
        )
        .unwrap()
        .is_match(line)
            && !line.contains('{')
        {
            return Pattern::FunctionStart;
        }

        // If statement
        if Regex::new(r"^\s*if\s*\(").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::IfStatement;
        }

        // For loop
        if Regex::new(r"^\s*for\s*\(").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::ForLoop;
        }

        // While loop
        if Regex::new(r"^\s*while\s*\(").unwrap().is_match(line) && !line.contains('{') {
            return Pattern::WhileLoop;
        }

        Pattern::Unknown
    }
}

/// Static pattern-based completion provider
pub struct StaticPatternProvider {
    detector: StaticPatternDetector,
    enabled: bool,
}

impl StaticPatternProvider {
    pub fn new() -> Self {
        Self {
            detector: StaticPatternDetector,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for StaticPatternProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CompletionProvider for StaticPatternProvider {
    async fn complete(&self, request: &CompletionRequest) -> Result<Vec<Completion>> {
        let pattern = self
            .detector
            .detect_pattern(&request.current_line, &request.language);

        if pattern == Pattern::Unknown {
            return Ok(Vec::new());
        }

        if let Some(template) = self.detector.get_template(pattern, &request.language) {
            let completion = Completion {
                text: template,
                cursor_offset: -2, // Move cursor inside the block
                confidence: 0.8,
                source: CompletionSource::Static,
                metadata: Some(serde_json::json!({
                    "pattern": format!("{:?}", pattern),
                })),
            };

            Ok(vec![completion])
        } else {
            Ok(Vec::new())
        }
    }

    fn name(&self) -> &str {
        "static-pattern"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_detection() {
        let detector = StaticPatternDetector;

        assert_eq!(
            detector.detect_pattern("fn main", "rust"),
            Pattern::FunctionStart
        );
        assert_eq!(
            detector.detect_pattern("pub fn test", "rust"),
            Pattern::FunctionStart
        );
        assert_eq!(
            detector.detect_pattern("async fn foo", "rust"),
            Pattern::FunctionStart
        );
        assert_eq!(
            detector.detect_pattern("fn bar() {", "rust"),
            Pattern::Unknown // Already has opening brace
        );
    }

    #[test]
    fn test_python_function_detection() {
        let detector = StaticPatternDetector;

        assert_eq!(
            detector.detect_pattern("def main", "python"),
            Pattern::FunctionStart
        );
        assert_eq!(
            detector.detect_pattern("async def test", "python"),
            Pattern::FunctionStart
        );
        assert_eq!(
            detector.detect_pattern("def foo():", "python"),
            Pattern::Unknown // Already has colon
        );
    }

    #[tokio::test]
    async fn test_static_provider() {
        let provider = StaticPatternProvider::new();
        let request = CompletionRequest {
            file_path: "test.rs".to_string(),
            language: "rust".to_string(),
            current_line: "fn main".to_string(),
            cursor_position: crate::completion::Position { line: 0, column: 7 },
            context_before: vec![],
            context_after: vec![],
        };

        let completions = provider.complete(&request).await.unwrap();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].source, CompletionSource::Static);
    }
}
