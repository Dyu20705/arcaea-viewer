use core::fmt;

use crate::{SourceMap, Span};

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Fatal parse or validation error.
    Error,
    /// Non-fatal warning.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => f.write_str("error"),
            Self::Warning => f.write_str("warning"),
        }
    }
}

/// Stable parser diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// Invalid lexical value, such as a malformed number.
    LexicalError,
    /// Syntax does not match the supported grammar.
    SyntaxError,
    /// Recognized but unsupported AFF event.
    UnsupportedEvent,
    /// Core domain constructor rejected a parsed value.
    DomainValidationError,
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexicalError => f.write_str("LEXICAL_ERROR"),
            Self::SyntaxError => f.write_str("SYNTAX_ERROR"),
            Self::UnsupportedEvent => f.write_str("UNSUPPORTED_EVENT"),
            Self::DomainValidationError => f.write_str("DOMAIN_VALIDATION_ERROR"),
        }
    }
}

/// Broad diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    /// Token or literal recognition failed.
    Lexical,
    /// Grammar shape did not match.
    Syntax,
    /// AFF event is outside this checkpoint's implemented subset.
    Unsupported,
    /// Parsed value failed core-domain validation.
    Domain,
}

/// Structured parser diagnostic with source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Source span.
    pub span: Span,
    /// Optional extra note.
    pub note: Option<String>,
    /// Optional suggested help.
    pub help: Option<String>,
    /// Broad category for callers.
    pub category: DiagnosticCategory,
}

impl Diagnostic {
    pub(crate) fn lexical(
        message: impl Into<String>,
        span: Span,
        note: Option<String>,
        help: Option<&'static str>,
    ) -> Self {
        Self {
            code: DiagnosticCode::LexicalError,
            severity: Severity::Error,
            message: message.into(),
            span,
            note,
            help: help.map(str::to_owned),
            category: DiagnosticCategory::Lexical,
        }
    }

    pub(crate) fn syntax(
        message: impl Into<String>,
        span: Span,
        note: Option<String>,
        help: Option<&'static str>,
    ) -> Self {
        Self {
            code: DiagnosticCode::SyntaxError,
            severity: Severity::Error,
            message: message.into(),
            span,
            note,
            help: help.map(str::to_owned),
            category: DiagnosticCategory::Syntax,
        }
    }

    pub(crate) fn unsupported(message: impl Into<String>, span: Span, note: String) -> Self {
        Self {
            code: DiagnosticCode::UnsupportedEvent,
            severity: Severity::Error,
            message: message.into(),
            span,
            note: Some(note),
            help: Some(
                "this checkpoint supports timing, tap, hold, arc, timinggroup, and arctap events"
                    .into(),
            ),
            category: DiagnosticCategory::Unsupported,
        }
    }

    pub(crate) fn domain_validation(
        message: impl Into<String>,
        span: Span,
        note: Option<String>,
        help: Option<&'static str>,
    ) -> Self {
        Self {
            code: DiagnosticCode::DomainValidationError,
            severity: Severity::Error,
            message: message.into(),
            span,
            note,
            help: help.map(str::to_owned),
            category: DiagnosticCategory::Domain,
        }
    }

    /// Renders this diagnostic with line, column, source excerpt, and caret.
    #[must_use]
    pub fn render(&self, source: &str, path: &str) -> String {
        let map = SourceMap::new(source);
        let location = map.line_column(self.span.start);
        let line_text = source
            .lines()
            .nth(location.line.saturating_sub(1))
            .unwrap_or_default();
        let caret_width = self.span.end.saturating_sub(self.span.start).max(1);
        let caret_padding = " ".repeat(location.column.saturating_sub(1));
        let carets = "^".repeat(caret_width.min(line_text.len().max(1)));

        let mut rendered = format!(
            "{severity}[{code}]: {message}\n --> {path}:{line}:{column}\n  |\n{line:>2} | {line_text}\n  | {caret_padding}{carets}",
            severity = self.severity,
            code = self.code,
            message = self.message,
            path = path,
            line = location.line,
            column = location.column,
        );

        if let Some(note) = &self.note {
            rendered.push_str("\n  = note: ");
            rendered.push_str(note);
        }
        if let Some(help) = &self.help {
            rendered.push_str("\n  = help: ");
            rendered.push_str(help);
        }

        rendered
    }
}
