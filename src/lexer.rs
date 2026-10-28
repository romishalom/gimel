//! Lexer.
//!
//! gimel source is stored in logical order, so lexing is the same left-to-right
//! (first character to last character) scan as for C. "Right to left" is a
//! property of how the text is displayed, not of how it is parsed.

use crate::diag::{quote, Diag, SourceFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ws,
    Newline,
    LineComment,
    BlockComment,
    Ident,
    Number,
    Str,
    Chr,
    Punct,
    Other,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, counted in characters.
    pub col: usize,
}

impl Token {
    pub fn is_trivia(&self) -> bool {
        matches!(self.kind, Kind::Ws | Kind::Newline | Kind::LineComment | Kind::BlockComment)
    }

    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

pub fn is_hebrew_letter(c: char) -> bool {
    ('\u{05D0}'..='\u{05EA}').contains(&c)
}

/// Niqqud, cantillation marks and other Hebrew points (U+0591..U+05C7).
pub fn is_hebrew_point(c: char) -> bool {
    ('\u{0591}'..='\u{05C7}').contains(&c)
}

pub fn is_final_form(c: char) -> bool {
    matches!(c, 'ך' | 'ם' | 'ן' | 'ף' | 'ץ')
}

/// Bidirectional control characters, forbidden anywhere in source text.
pub fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

const PUNCT3: &[&str] = &["...", "<<=", ">>="];
const PUNCT2: &[&str] = &[
    "->", "++", "--", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=", "##", "::",
];
const PUNCT1: &str = "[](){}.&*+-~!/%<>^|?:;=,#\\";

struct Lexer<'a> {
    sf: &'a SourceFile,
    chars: Vec<char>,
    i: usize,
    line: usize,
    col: usize,
    lenient: bool,
    toks: Vec<Token>,
    diags: Vec<Diag>,
}

/// Tokenize `sf`. With `lenient` set (used for C input) any Unicode alphanumeric
/// is accepted in identifiers and unknown characters are passed through.
pub fn lex(sf: &SourceFile, lenient: bool) -> (Vec<Token>, Vec<Diag>) {
    let mut lx = Lexer {
        sf,
        chars: sf.text.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
        lenient,
        toks: Vec::new(),
        diags: Vec::new(),
    };
    lx.forbid_bidi_controls();
    lx.run();
    (lx.toks, lx.diags)
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.chars.get(self.i + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.i).copied()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn push(&mut self, kind: Kind, text: String, line: usize, col: usize) {
        self.toks.push(Token { kind, text, line, col });
    }

    fn forbid_bidi_controls(&mut self) {
        let (mut line, mut col) = (1usize, 1usize);
        for &c in &self.chars {
            if is_bidi_control(c) {
                let d = self
                    .sf
                    .error(
                        "G001",
                        line,
                        col,
                        1,
                        format!("bidirectional control character U+{:04X} is forbidden in gimel source", c as u32),
                    )
                    .with_hint("bidi controls can make code display differently from how it parses; inside string literals use an escape such as \\u202E");
                self.diags.push(d);
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
    }

    fn ident_start(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || is_hebrew_letter(c) || (self.lenient && !c.is_ascii() && c.is_alphabetic())
    }

    fn ident_continue(&self, c: char) -> bool {
        self.ident_start(c) || c.is_ascii_digit() || (self.lenient && !c.is_ascii() && c.is_alphanumeric())
    }

    fn run(&mut self) {
        while let Some(c) = self.peek() {
            let (line, col) = (self.line, self.col);
            match c {
                '\n' => {
                    self.advance();
                    self.push(Kind::Newline, "\n".into(), line, col);
                }
                '\r' => {
                    self.advance();
                    if self.peek() == Some('\n') {
                        self.advance();
                        self.push(Kind::Newline, "\r\n".into(), line, col);
                    } else {
                        // A lone CR is a line terminator too.
                        self.line += 1;
                        self.col = 1;
                        self.push(Kind::Newline, "\r".into(), line, col);
                    }
                }
                ' ' | '\t' | '\u{0B}' | '\u{0C}' => {
                    let mut s = String::new();
                    while let Some(w) = self.peek() {
                        if matches!(w, ' ' | '\t' | '\u{0B}' | '\u{0C}') {
                            s.push(w);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.push(Kind::Ws, s, line, col);
                }
                '/' if self.peek_at(1) == Some('/') => {
                    let mut s = String::new();
                    while let Some(w) = self.peek() {
                        if w == '\n' || w == '\r' {
                            break;
                        }
                        s.push(w);
                        self.advance();
                    }
                    self.push(Kind::LineComment, s, line, col);
                }
                '/' if self.peek_at(1) == Some('*') => self.block_comment(line, col),
                '"' => self.quoted('"', String::new(), Kind::Str, line, col),
                '\'' => self.quoted('\'', String::new(), Kind::Chr, line, col),
                d if d.is_ascii_digit() => self.number(line, col),
                '.' if self.peek_at(1).map_or(false, |n| n.is_ascii_digit()) => self.number(line, col),
                c if self.ident_start(c) => self.ident(line, col),
                c if is_bidi_control(c) => {
                    // Already diagnosed by the pre-pass; keep scanning.
                    self.advance();
                    self.push(Kind::Ws, String::new(), line, col);
                }
                _ => self.punct(c, line, col),
            }
        }
    }

    fn block_comment(&mut self, line: usize, col: usize) {
        let mut s = String::new();
        s.push(self.advance().unwrap());
        s.push(self.advance().unwrap());
        loop {
            match self.peek() {
                None => {
                    let d = self.sf.error("G008", line, col, 2, "unterminated block comment".into());
                    self.diags.push(d);
                    break;
                }
                Some('*') if self.peek_at(1) == Some('/') => {
                    s.push(self.advance().unwrap());
                    s.push(self.advance().unwrap());
                    break;
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        self.push(Kind::BlockComment, s, line, col);
    }

    fn quoted(&mut self, quote_ch: char, prefix: String, kind: Kind, line: usize, col: usize) {
        let mut s = prefix;
        s.push(self.advance().unwrap());
        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => {
                    let what = if quote_ch == '"' { "string" } else { "character" };
                    let d = self.sf.error("G008", line, col, s.chars().count(), format!("unterminated {what} literal"));
                    self.diags.push(d);
                    break;
                }
                Some('\\') => {
                    s.push(self.advance().unwrap());
                    match self.peek() {
                        Some('\r') => {
                            s.push(self.advance().unwrap());
                            if self.peek() == Some('\n') {
                                s.push(self.advance().unwrap());
                            }
                        }
                        Some(_) => s.push(self.advance().unwrap()),
                        None => {}
                    }
                }
                Some(c) if c == quote_ch => {
                    s.push(self.advance().unwrap());
                    break;
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        self.push(kind, s, line, col);
    }

    fn number(&mut self, line: usize, col: usize) {
        let mut s = String::new();
        let mut prev = '\0';
        while let Some(c) = self.peek() {
            let ok = c.is_ascii_alphanumeric() || c == '_' || c == '.';
            if ok {
                s.push(c);
                prev = c;
                self.advance();
            } else if (c == '+' || c == '-') && matches!(prev, 'e' | 'E' | 'p' | 'P') {
                s.push(c);
                prev = c;
                self.advance();
            } else if c == '\'' && prev.is_ascii_alphanumeric() && self.peek_at(1).map_or(false, |n| n.is_ascii_alphanumeric() || n == '_') {
                // C23 digit separator.
                s.push(c);
                prev = c;
                self.advance();
            } else {
                break;
            }
        }
        if let Some(c) = self.peek() {
            if self.ident_start(c) && !c.is_ascii() {
                let mut extra = String::new();
                while let Some(w) = self.peek() {
                    if self.ident_continue(w) {
                        extra.push(w);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let d = self
                    .sf
                    .error(
                        "G009",
                        line,
                        col,
                        s.chars().count() + extra.chars().count(),
                        format!("invalid suffix {} on numeric literal", quote(&extra)),
                    )
                    .with_hint("numeric literals use ASCII digits and C suffixes; Hebrew numerals (gematria) are not accepted");
                self.diags.push(d);
                s.push_str(&extra);
            }
        }
        self.push(Kind::Number, s, line, col);
    }

    fn ident(&mut self, line: usize, col: usize) {
        let mut s = String::new();
        let mut points = false;
        while let Some(c) = self.peek() {
            if self.ident_continue(c) {
                s.push(c);
                self.advance();
            } else if is_hebrew_point(c) && c != '\u{05BE}' && !self.lenient {
                points = true;
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if points {
            let d = self
                .sf
                .error("G003", line, col, s.chars().count(), format!("Hebrew points (niqqud, cantillation) are not allowed in identifiers or keywords: {}", quote(&s)))
                .with_hint("points are allowed inside string literals and comments");
            self.diags.push(d);
        }
        // Encoding prefixes of string and character literals.
        if matches!(s.as_str(), "u8" | "u" | "U" | "L") {
            match self.peek() {
                Some('"') => return self.quoted('"', s, Kind::Str, line, col),
                Some('\'') => return self.quoted('\'', s, Kind::Chr, line, col),
                _ => {}
            }
        }
        self.push(Kind::Ident, s, line, col);
    }

    fn punct(&mut self, c: char, line: usize, col: usize) {
        let rest: String = self.chars[self.i..].iter().take(3).collect();
        for p in PUNCT3 {
            if rest.starts_with(p) {
                for _ in 0..3 {
                    self.advance();
                }
                return self.push(Kind::Punct, (*p).into(), line, col);
            }
        }
        for p in PUNCT2 {
            if rest.starts_with(p) {
                for _ in 0..2 {
                    self.advance();
                }
                return self.push(Kind::Punct, (*p).into(), line, col);
            }
        }
        if PUNCT1.contains(c) {
            self.advance();
            return self.push(Kind::Punct, c.to_string(), line, col);
        }
        self.advance();
        if !self.lenient {
            let d = match c {
                '\u{05BE}' | '\u{05F3}' | '\u{05F4}' => self
                    .sf
                    .error("G010", line, col, 1, format!("Hebrew punctuation {} (U+{:04X}) is not a delimiter or operator", quote(&c.to_string()), c as u32))
                    .with_hint("use ASCII quotes for strings and characters; maqaf, geresh and gershayim are not part of the language"),
                c if is_hebrew_point(c) => self.sf.error("G002", line, col, 1, format!("unexpected Hebrew point U+{:04X}", c as u32)),
                c => self.sf.error("G002", line, col, 1, format!("unexpected character {} (U+{:04X})", quote(&c.to_string()), c as u32)),
            };
            self.diags.push(d);
        }
        self.push(Kind::Other, c.to_string(), line, col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(Kind, String)> {
        let sf = SourceFile::new("t", src);
        let (t, d) = lex(&sf, false);
        assert!(d.is_empty(), "{d:?}");
        t.into_iter().filter(|t| !t.is_trivia()).map(|t| (t.kind, t.text)).collect()
    }

    #[test]
    fn basic_tokens() {
        let t = kinds("שלם א = 0x1F + 'x'; // הערה");
        let texts: Vec<_> = t.iter().map(|x| x.1.as_str()).collect();
        assert_eq!(texts, ["שלם", "א", "=", "0x1F", "+", "'x'", ";"]);
    }

    #[test]
    fn prefixed_literals() {
        let t = kinds("u8\"x\" L'a'");
        assert_eq!(t[0], (Kind::Str, "u8\"x\"".into()));
        assert_eq!(t[1], (Kind::Chr, "L'a'".into()));
    }

    #[test]
    fn digit_separators_and_exponents() {
        let t = kinds("1'000'000 1e+5 0x1p-3");
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn bidi_controls_are_rejected() {
        let sf = SourceFile::new("t", "\"a\u{202E}b\"");
        let (_, d) = lex(&sf, false);
        assert!(d.iter().any(|d| d.code == "G001"));
    }

    #[test]
    fn niqqud_in_identifier_is_rejected_but_fine_in_strings() {
        let sf = SourceFile::new("t", "שָׁלוֹם");
        let (_, d) = lex(&sf, false);
        assert!(d.iter().any(|d| d.code == "G003"));
        let sf = SourceFile::new("t", "\"שָׁלוֹם\" // שָׁלוֹם");
        let (_, d) = lex(&sf, false);
        assert!(d.is_empty());
    }

    #[test]
    fn hebrew_punctuation_is_rejected() {
        let sf = SourceFile::new("t", "א״ב");
        let (_, d) = lex(&sf, false);
        assert!(d.iter().any(|d| d.code == "G010"));
    }

    #[test]
    fn gematria_suffix_is_rejected() {
        let sf = SourceFile::new("t", "5א");
        let (_, d) = lex(&sf, false);
        assert!(d.iter().any(|d| d.code == "G009"));
    }
}
