//! Identifier mangling.
//!
//! Hebrew identifiers can be passed to the C compiler as UTF-8 (every modern
//! clang and gcc accepts them, and ELF and Mach-O symbol tables hold arbitrary
//! bytes), or mangled to ASCII for toolchains that do not.
//!
//! The ASCII form is `gml__` followed by the identifier where ASCII letters and
//! digits are kept, `_` becomes `_u` and every other character becomes `_xHHHH`
//! (its code point as four hex digits). The mapping is injective and reversible.
//! Because `gml__` is reserved, a user identifier can never collide with it.

pub const PREFIX: &str = "gml__";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mangle {
    Utf8,
    Ascii,
}

impl Mangle {
    /// UTF-8 everywhere except Windows, where ASCII is safer.
    pub fn platform_default() -> Mangle {
        if cfg!(windows) {
            Mangle::Ascii
        } else {
            Mangle::Utf8
        }
    }

    pub fn parse(s: &str) -> Option<Mangle> {
        match s {
            "utf8" | "utf-8" => Some(Mangle::Utf8),
            "ascii" => Some(Mangle::Ascii),
            _ => None,
        }
    }
}

pub fn mangle(id: &str, mode: Mangle) -> String {
    if mode == Mangle::Utf8 || id.is_ascii() {
        return id.to_string();
    }
    let mut s = String::from(PREFIX);
    for c in id.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else if c == '_' {
            s.push_str("_u");
        } else {
            s.push_str(&format!("_x{:04X}", c as u32));
        }
    }
    s
}

pub fn demangle(s: &str) -> Option<String> {
    let rest = s.strip_prefix(PREFIX)?;
    let cs: Vec<char> = rest.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '_' {
            match cs.get(i + 1)? {
                'u' => {
                    out.push('_');
                    i += 2;
                }
                'x' => {
                    let hex: String = cs.get(i + 2..i + 6)?.iter().collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                    i += 6;
                }
                _ => return None,
            }
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_untouched() {
        assert_eq!(mangle("printf", Mangle::Ascii), "printf");
    }

    #[test]
    fn utf8_mode_is_identity() {
        assert_eq!(mangle("סך", Mangle::Utf8), "סך");
    }

    #[test]
    fn roundtrip() {
        for id in ["סך", "א1", "מערך_ב", "x_ש", "שלום123"] {
            let m = mangle(id, Mangle::Ascii);
            assert!(m.is_ascii() && m.starts_with(PREFIX), "{m}");
            assert_eq!(demangle(&m).as_deref(), Some(id));
        }
    }

    #[test]
    fn distinct_inputs_stay_distinct() {
        assert_ne!(mangle("א_", Mangle::Ascii), mangle("אu", Mangle::Ascii));
    }
}
