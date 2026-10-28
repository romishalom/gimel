//! The gimel keyword tables.
//!
//! This module is the single source of truth for the translator, the reverse
//! (C to gimel) translator and the `gimel keywords` command. It mirrors the
//! keyword table of the language specification.

/// One gimel keyword or preprocessor directive.
#[derive(Debug, Clone, Copy)]
pub struct Keyword {
    pub category: &'static str,
    /// The gimel spelling (without a leading `#` for directives).
    pub gimel: &'static str,
    /// All C spellings. The first one is what the translator emits.
    pub c: &'static [&'static str],
    pub pron: &'static str,
    pub sense: &'static str,
}

macro_rules! kw {
    ($cat:expr, $g:expr, [$($c:expr),+], $p:expr, $s:expr) => {
        Keyword { category: $cat, gimel: $g, c: &[$($c),+], pron: $p, sense: $s }
    };
}

pub const KEYWORDS: &[Keyword] = &[
    // Control flow
    kw!("Control flow", "בדוק", ["if"], "bdok", "check"),
    kw!("Control flow", "אחרת", ["else"], "acheret", "[act] otherwise; elliptical imperative"),
    kw!("Control flow", "בחר", ["switch"], "bchar", "choose"),
    kw!("Control flow", "טפל", ["case"], "tapel", "handle"),
    kw!("Control flow", "השלם", ["default"], "hashlem", "complete (cover what remains)"),
    kw!("Control flow", "עבור", ["for"], "avor", "traverse; also the Hebrew word for \"for\""),
    kw!("Control flow", "המשך", ["while"], "hamshekh", "continue (as long as)"),
    kw!("Control flow", "בצע", ["do"], "batze'a", "perform"),
    kw!("Control flow", "עצור", ["break"], "atzor", "stop"),
    kw!("Control flow", "דלג", ["continue"], "daleg", "skip (to the next iteration)"),
    kw!("Control flow", "קפוץ", ["goto"], "kfotz", "jump"),
    kw!("Control flow", "החזר", ["return"], "hachzer", "return"),
    // Types
    kw!("Types", "ריק", ["void"], "rek", "empty"),
    kw!("Types", "תו", ["char"], "tav", "character"),
    kw!("Types", "קצר", ["short"], "katzar", "short"),
    kw!("Types", "שלם", ["int"], "shalem", "whole"),
    kw!("Types", "ארוך", ["long"], "arokh", "long"),
    kw!("Types", "צף", ["float"], "tzaf", "floating"),
    kw!("Types", "כפול", ["double"], "kaful", "double"),
    kw!("Types", "מסומן", ["signed"], "mesuman", "signed"),
    kw!("Types", "ללס", ["unsigned"], "lelas", "acronym of ללא סימן, \"without sign\""),
    kw!("Types", "בוליאני", ["_Bool", "bool"], "buli'ani", "Boolean"),
    kw!("Types", "מרוכב", ["_Complex"], "merukav", "complex"),
    kw!("Types", "מדומה", ["_Imaginary"], "medume", "imaginary"),
    kw!("Types", "סיבית", ["_BitInt"], "sibit", "bit; written סיבית(24)"),
    kw!("Types", "עשרוני32", ["_Decimal32"], "asroni", "decimal"),
    kw!("Types", "עשרוני64", ["_Decimal64"], "asroni", "decimal"),
    kw!("Types", "עשרוני128", ["_Decimal128"], "asroni", "decimal"),
    // Aggregates
    kw!("Aggregates", "בנה", ["struct"], "bneh", "build"),
    kw!("Aggregates", "אחד", ["union"], "ached", "unify"),
    kw!("Aggregates", "מנה", ["enum"], "mneh", "count, enumerate"),
    kw!("Aggregates", "כנה", ["typedef"], "kaneh", "give a nickname, alias"),
    // Type operators
    kw!("Type operators", "מדוד", ["sizeof"], "madod", "measure"),
    kw!("Type operators", "מדש", ["_Alignof", "alignof"], "medash", "acronym of מדוד ישור, \"measure alignment\""),
    kw!("Type operators", "ישר", ["_Alignas", "alignas"], "yasher", "align"),
    kw!("Type operators", "סווג", ["typeof"], "saveg", "classify"),
    kw!("Type operators", "סווגחשוף", ["typeof_unqual"], "saveg-chashuf", "classify bare (fused compound)"),
    kw!("Type operators", "התאם", ["_Generic"], "hatem", "match (by type)"),
    kw!("Type operators", "הסק", ["auto"], "hesek", "deduce"),
    // Storage and qualifiers
    kw!("Storage and qualifiers", "שמור", ["static"], "shmor", "keep (persist)"),
    kw!("Storage and qualifiers", "ייבא", ["extern"], "yabe", "import"),
    kw!("Storage and qualifiers", "רשום", ["register"], "rshom", "register, record"),
    kw!("Storage and qualifiers", "שלב", ["inline"], "shalev", "integrate"),
    kw!("Storage and qualifiers", "קבע", ["const"], "kva", "fix, set firmly"),
    kw!("Storage and qualifiers", "רענן", ["volatile"], "ra'anen", "refresh (always re-read)"),
    kw!("Storage and qualifiers", "הגבל", ["restrict"], "hagbel", "restrict"),
    kw!("Storage and qualifiers", "אטום", ["_Atomic"], "atom", "seal; echoes אטומי, \"atomic\""),
    kw!("Storage and qualifiers", "בודד", ["_Thread_local", "thread_local"], "boded", "isolate"),
    kw!("Storage and qualifiers", "חשב", ["constexpr"], "chashev", "compute (at translation time)"),
    kw!("Storage and qualifiers", "אלתחזור", ["_Noreturn"], "al-tachzor", "\"do not return\" (fused negative imperative)"),
    kw!("Storage and qualifiers", "וודא", ["_Static_assert", "static_assert"], "vadeh", "ensure, verify"),
    // Literals
    kw!("Literals", "אמת", ["true"], "emet", "truth"),
    kw!("Literals", "שקר", ["false"], "sheker", "falsehood"),
    kw!("Literals", "כלום", ["nullptr"], "klum", "nothing"),
    // Entry point and operators
    kw!("Entry point", "התחל", ["main"], "hatchel", "start"),
    kw!("Operators", "מוגדר", ["defined"], "mugdar", "defined (predicate, not a command)"),
];

pub const DIRECTIVES: &[Keyword] = &[
    kw!("Preprocessor", "כלול", ["include"], "klol", "include"),
    kw!("Preprocessor", "הגדר", ["define"], "hagder", "define"),
    kw!("Preprocessor", "בטל", ["undef"], "batel", "cancel"),
    kw!("Preprocessor", "בדוק", ["if"], "bdok", "check"),
    kw!("Preprocessor", "בדוקהוגדר", ["ifdef"], "bdok-hugdar", "check [it is] defined"),
    kw!("Preprocessor", "בדוקחסר", ["ifndef"], "bdok-chaser", "check [it is] missing"),
    kw!("Preprocessor", "אחרתבדוק", ["elif"], "acheret-bdok", "otherwise check"),
    kw!("Preprocessor", "אחרתבדוקהוגדר", ["elifdef"], "acheret-bdok-hugdar", "otherwise check defined"),
    kw!("Preprocessor", "אחרתבדוקחסר", ["elifndef"], "acheret-bdok-chaser", "otherwise check missing"),
    kw!("Preprocessor", "אחרת", ["else"], "acheret", "otherwise"),
    kw!("Preprocessor", "סיים", ["endif"], "sayem", "finish"),
    kw!("Preprocessor", "היכשל", ["error"], "hikashel", "fail"),
    kw!("Preprocessor", "התרע", ["warning"], "hatre'a", "warn"),
    kw!("Preprocessor", "כוון", ["pragma"], "kaven", "direct, steer"),
    kw!("Preprocessor", "מספר", ["line"], "masper", "number"),
    kw!("Preprocessor", "הטבע", ["embed"], "hatbe'a", "imprint, embed"),
];

/// Look up a gimel keyword (not a directive name).
pub fn keyword(gimel: &str) -> Option<&'static Keyword> {
    KEYWORDS.iter().find(|k| k.gimel == gimel)
}

/// Look up the gimel keyword for a C spelling.
pub fn keyword_from_c(c: &str) -> Option<&'static Keyword> {
    KEYWORDS.iter().find(|k| k.c.contains(&c))
}

/// Look up a gimel directive name (the word after `#`).
pub fn directive(gimel: &str) -> Option<&'static Keyword> {
    DIRECTIVES.iter().find(|k| k.gimel == gimel)
}

/// Look up the gimel directive for a C directive name.
pub fn directive_from_c(c: &str) -> Option<&'static Keyword> {
    DIRECTIVES.iter().find(|k| k.c.contains(&c))
}

/// One printable row of the keyword table.
#[derive(Debug, Clone)]
pub struct Row {
    pub category: String,
    pub c: String,
    pub gimel: String,
    pub pron: String,
    pub sense: String,
}

pub fn rows() -> Vec<Row> {
    let mut v = Vec::new();
    for k in KEYWORDS {
        v.push(Row {
            category: k.category.into(),
            c: k.c.join(", "),
            gimel: k.gimel.into(),
            pron: k.pron.into(),
            sense: k.sense.into(),
        });
    }
    for k in DIRECTIVES {
        v.push(Row {
            category: k.category.into(),
            c: k.c.iter().map(|c| format!("#{c}")).collect::<Vec<_>>().join(", "),
            gimel: format!("#{}", k.gimel),
            pron: k.pron.into(),
            sense: k.sense.into(),
        });
    }
    v
}

fn width(s: &str) -> usize {
    s.chars().count()
}

/// Aligned plain-text table.
pub fn render_table() -> String {
    let rows = rows();
    let head = ["Category", "C", "gimel", "Pronounced", "Sense"];
    let mut w = head.map(width);
    for r in &rows {
        for (i, cell) in [&r.category, &r.c, &r.gimel, &r.pron, &r.sense].iter().enumerate() {
            w[i] = w[i].max(width(cell));
        }
    }
    let line = |cells: [&str; 5]| -> String {
        let mut s = String::new();
        for (i, c) in cells.iter().enumerate() {
            s.push_str(c);
            if i < 4 {
                s.push_str(&" ".repeat(w[i] - width(c) + 2));
            }
        }
        s.push('\n');
        s
    };
    let mut out = line(head);
    out.push_str(&"-".repeat(w.iter().sum::<usize>() + 8));
    out.push('\n');
    for r in &rows {
        out.push_str(&line([&r.category, &r.c, &r.gimel, &r.pron, &r.sense]));
    }
    out
}

/// Markdown table, as used in the language specification.
pub fn render_markdown() -> String {
    let mut out = String::from("| Category | C | gimel | Pronounced | Sense |\n|---|---|---|---|---|\n");
    for r in rows() {
        let c = r.c.split(", ").map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("| {} | {} | **{}** | {} | {} |\n", r.category, c, r.gimel, r.pron, r.sense));
    }
    out
}

/// Tab-separated values.
pub fn render_tsv() -> String {
    let mut out = String::from("category\tc\tgimel\tpronounced\tsense\n");
    for r in rows() {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\n", r.category, r.c, r.gimel, r.pron, r.sense));
    }
    out
}

fn json_escape(s: &str) -> String {
    let mut o = String::new();
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o
}

/// JSON array of objects.
pub fn render_json() -> String {
    let rows = rows();
    let mut out = String::from("[\n");
    for (i, r) in rows.iter().enumerate() {
        out.push_str(&format!(
            "  {{\"category\": \"{}\", \"c\": \"{}\", \"gimel\": \"{}\", \"pronounced\": \"{}\", \"sense\": \"{}\"}}{}\n",
            json_escape(&r.category),
            json_escape(&r.c),
            json_escape(&r.gimel),
            json_escape(&r.pron),
            json_escape(&r.sense),
            if i + 1 < rows.len() { "," } else { "" }
        ));
    }
    out.push_str("]\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_underscores_in_gimel_keywords() {
        for k in KEYWORDS.iter().chain(DIRECTIVES) {
            assert!(!k.gimel.contains('_'), "{}", k.gimel);
        }
    }

    #[test]
    fn gimel_keywords_are_unique() {
        for (i, a) in KEYWORDS.iter().enumerate() {
            for b in &KEYWORDS[i + 1..] {
                assert_ne!(a.gimel, b.gimel);
            }
        }
        for (i, a) in DIRECTIVES.iter().enumerate() {
            for b in &DIRECTIVES[i + 1..] {
                assert_ne!(a.gimel, b.gimel);
            }
        }
    }

    #[test]
    fn lookups() {
        assert_eq!(keyword("שלם").unwrap().c[0], "int");
        assert_eq!(keyword_from_c("bool").unwrap().gimel, "בוליאני");
        assert_eq!(directive("כלול").unwrap().c[0], "include");
        assert_eq!(directive_from_c("elifdef").unwrap().gimel, "אחרתבדוקהוגדר");
    }
}
