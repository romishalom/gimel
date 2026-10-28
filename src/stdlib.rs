//! Standard headers and library names.
//!
//! The first four headers and the twelve function names come from the
//! informative section of the specification. The remaining headers are
//! implementation extensions. A function name becomes available through a
//! function-like macro when its header is included, so a variable that happens
//! to share a name is not disturbed.

pub struct Header {
    pub gimel: &'static str,
    pub c: &'static str,
    /// (gimel name, C name)
    pub funcs: &'static [(&'static str, &'static str)],
}

pub const HEADERS: &[Header] = &[
    Header { gimel: "קלטפלט", c: "stdio.h", funcs: &[("הדפס", "printf"), ("קלוט", "scanf"), ("פתח", "fopen"), ("סגור", "fclose")] },
    Header { gimel: "כללי", c: "stdlib.h", funcs: &[("הקצה", "malloc"), ("שחרר", "free"), ("מיין", "qsort"), ("צא", "exit")] },
    Header { gimel: "מחרוזת", c: "string.h", funcs: &[("העתק", "memcpy"), ("מלא", "memset"), ("השווה", "strcmp"), ("ספור", "strlen")] },
    Header { gimel: "חשבון", c: "math.h", funcs: &[] },
    // Implementation extensions:
    Header { gimel: "טענות", c: "assert.h", funcs: &[("טען", "assert")] },
    Header { gimel: "זמן", c: "time.h", funcs: &[] },
    Header { gimel: "תווים", c: "ctype.h", funcs: &[] },
    Header { gimel: "גבולות", c: "limits.h", funcs: &[] },
    Header { gimel: "שלמים", c: "stdint.h", funcs: &[] },
    Header { gimel: "הגדרות", c: "stddef.h", funcs: &[] },
    Header { gimel: "שגיאות", c: "errno.h", funcs: &[] },
];

pub fn header_by_gimel(name: &str) -> Option<&'static Header> {
    HEADERS.iter().find(|h| h.gimel == name)
}

pub fn header_by_c(name: &str) -> Option<&'static Header> {
    HEADERS.iter().find(|h| h.c == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookups() {
        assert_eq!(header_by_gimel("קלטפלט").unwrap().c, "stdio.h");
        assert_eq!(header_by_c("string.h").unwrap().gimel, "מחרוזת");
        assert!(header_by_gimel("nope").is_none());
    }

    #[test]
    fn library_names_have_no_underscores() {
        for h in HEADERS {
            assert!(!h.gimel.contains('_'));
            for (g, _) in h.funcs {
                assert!(!g.contains('_'));
            }
        }
    }
}
