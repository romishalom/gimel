//! Diagnostics and source files.

use std::sync::atomic::{AtomicBool, Ordering};

static ISOLATE: AtomicBool = AtomicBool::new(false);

/// When enabled, quoted non-ASCII fragments in messages are wrapped in Unicode
/// first-strong isolates (U+2068 .. U+2069) so mixed Hebrew/ASCII text keeps its
/// reading order in bidi-aware terminals. Isolates are allowed in tool output
/// but are forbidden in source text.
pub fn set_isolate(on: bool) {
    ISOLATE.store(on, Ordering::Relaxed);
}

/// Quote a fragment for use inside a message.
pub fn quote(s: &str) -> String {
    if ISOLATE.load(Ordering::Relaxed) && !s.is_ascii() {
        format!("`\u{2068}{s}\u{2069}`")
    } else {
        format!("`{s}`")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diag {
    pub severity: Severity,
    pub code: &'static str,
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub len: usize,
    pub message: String,
    pub hint: Option<String>,
    pub line_text: String,
}

impl Diag {
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// A source file held in memory. The BOM, if any, is dropped.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text: String = text.into();
        let text = match text.strip_prefix('\u{feff}') {
            Some(t) => t.to_string(),
            None => text,
        };
        SourceFile { name: name.into(), text }
    }

    pub fn line_text(&self, line: usize) -> String {
        self.text
            .split('\n')
            .nth(line.saturating_sub(1))
            .map(|l| l.trim_end_matches('\r').to_string())
            .unwrap_or_default()
    }

    pub fn diag(&self, severity: Severity, code: &'static str, line: usize, col: usize, len: usize, message: String) -> Diag {
        Diag {
            severity,
            code,
            file: self.name.clone(),
            line,
            col,
            len: len.max(1),
            message,
            hint: None,
            line_text: self.line_text(line),
        }
    }

    pub fn error(&self, code: &'static str, line: usize, col: usize, len: usize, message: String) -> Diag {
        self.diag(Severity::Error, code, line, col, len, message)
    }

    pub fn warning(&self, code: &'static str, line: usize, col: usize, len: usize, message: String) -> Diag {
        self.diag(Severity::Warning, code, line, col, len, message)
    }
}

/// Render one diagnostic in a rustc-like layout.
pub fn render(d: &Diag, color: bool) -> String {
    let (label, hue) = match d.severity {
        Severity::Error => ("error", "\x1b[1;31m"),
        Severity::Warning => ("warning", "\x1b[1;33m"),
    };
    let (bold, blue, reset) = if color { ("\x1b[1m", "\x1b[1;34m", "\x1b[0m") } else { ("", "", "") };
    let hue = if color { hue } else { "" };
    let mut s = String::new();
    s.push_str(&format!("{hue}{label}[{}]{reset}{bold}: {}{reset}\n", d.code, d.message));
    let w = d.line.to_string().len();
    s.push_str(&format!("{:>w$}{blue}-->{reset} {}:{}:{}\n", "", d.file, d.line, d.col, w = w + 1));
    if !d.line_text.is_empty() {
        s.push_str(&format!("{:>w$} {blue}|{reset}\n", "", w = w));
        s.push_str(&format!("{blue}{:>w$} |{reset} {}\n", d.line, d.line_text, w = w));
        let pad: String = d
            .line_text
            .chars()
            .take(d.col.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        s.push_str(&format!("{:>w$} {blue}|{reset} {pad}{hue}{}{reset}\n", "", "^".repeat(d.len), w = w));
    }
    if let Some(h) = &d.hint {
        s.push_str(&format!("{:>w$} {blue}= help:{reset} {}\n", "", h, w = w));
    }
    s
}

/// Print diagnostics to stderr, with color when stderr is a terminal.
pub fn print_all(diags: &[Diag]) {
    use std::io::IsTerminal;
    let color = std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    for d in diags {
        eprintln!("{}", render(d, color));
    }
}
