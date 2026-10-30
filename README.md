# gimel (ג)

C in Hebrew. gimel is C23 with Hebrew keywords written as commands, stored in logical
order and displayed right to left. This repository is a Rust implementation: a
translator (gimel to C, and C back to gimel) plus a CLI that drives **clang**
(or gcc, or any C compiler) for code generation. It has no dependencies beyond the
Rust standard library.

```
$ cat examples/hello.ג
#כלול <קלטפלט>

שלם התחל(ריק) {
    הדפס("שלום, עולם!\n");
    החזר 0;
}

$ gimel run examples/hello.ג
שלום, עולם!
```

## Files

| Extension | Meaning |
|---|---|
| `.ג` | gimel source |
| `.כ` | gimel header (כ for כותרת); `.ך` is accepted as an alias |
| `.gml`, `.gmlh` | ASCII aliases for source and header |

Source files are UTF-8 in **logical order** (the order you read them). The right-to-left
look, including mirrored `(` `)`, `[` `]`, `{` `}` and `<` `>`, comes from Unicode
bidi mirroring in your editor, so the compiler uses the same delimiter code points as C.

## Install and build

```
cargo build --release          # binary at target/release/gimel
cargo test                     # unit tests and CLI end-to-end tests (compile and run with clang/gcc)
```

You need a C compiler to produce executables. `gimel` looks for one in this order:
`--cc`, `$GIMEL_CC`, `$CC`, then `clang`, `cc`, `gcc`. `check`, `emit-c`, `from-c` and
`keywords` need no compiler.

## Commands

| Command | What it does |
|---|---|
| `gimel build a.ג b.ג -o prog` | Translate, then compile and link. Several sources are allowed. |
| `gimel run a.ג [-- args]` | Build in a temporary directory and run. The program's exit code is returned. |
| `gimel emit-c a.ג` | Print the generated C. `--out-dir DIR` writes the whole include tree. |
| `gimel check a.ג` | Report diagnostics and produce nothing. |
| `gimel from-c a.c` | Translate C into gimel. |
| `gimel keywords --format table\|markdown\|json\|tsv` | Print the keyword table. |

Common options: `-o`, `-I dir`, `-D name=value`, `-O0..-O3/-Os`, `-g`, `-c`, `--cc cmd`,
`--std c2x` (default), `--keep dir` (keep the generated C), `--no-line`,
`--mangle utf8|ascii`, `--warn-mixed`, `-Werror`, `--isolate`. Everything after `--` is
passed to the C compiler (for `run`, to the program).

## How it works

1. The lexer reads logical-order UTF-8 and rejects what the spec forbids: bidi control
   characters anywhere (G001), Hebrew points in identifiers (G003), mixed-script
   identifiers (G004), Hebrew punctuation as operators (G010), Hebrew numerals (G009).
2. The translator substitutes keywords and directives from `src/keywords.rs`, preserving
   whitespace, comments and line breaks, then emits `#line 1 "file.ג"`. Compiler errors
   therefore point at your gimel source, with correct line numbers.
3. Quoted `#כלול "x.כ"` headers are found, translated to `x.h` in a mirrored work
   directory, and compiled alongside. Angle-bracket standard headers such as
   `<קלטפלט>` map to `<stdio.h>`; other headers pass through unchanged.
4. The work directory is handed to clang with `-std=c2x`.

Notes on the mapping:

- `בוליאני` becomes `_Bool` (works in every mode); `אמת`, `שקר`, `כלום` become `true`,
  `false`, `nullptr`, which need a C23-capable compiler (clang 17+, gcc 13+).
- `התחל` is `main`. `אחרת בדוק` is `else if`. `ללס` is `unsigned`; `מדש` is `_Alignof`.
- C keywords written in ASCII (`int`, `#include`, `main`) are errors with a hint naming
  the gimel spelling (G006, G007). Other ASCII identifiers such as `foo` are fine.
- Library names are function-like macros defined when their header is included:
  `הדפס` `קלוט` `פתח` `סגור` (`<קלטפלט>`), `הקצה` `שחרר` `מיין` `צא` (`<כללי>`),
  `העתק` `מלא` `השווה` `ספור` (`<מחרוזת>`), `טען` (`<טענות>`). Headers `<חשבון>`, `<זמן>`,
  `<תווים>`, `<גבולות>`, `<שלמים>`, `<הגדרות>`, `<שגיאות>` are also mapped.
- `[[הוצא("name")]]` before a function or variable declaration sets its symbol name
  (lowered to `#pragma redefine_extname`), for linking with C. The name must be an
  ASCII identifier.
- Hebrew identifiers are passed to the C compiler as UTF-8 by default. `--mangle ascii`
  encodes them as `gml__…` (reversible; `gml__` is reserved), the default on Windows.

Warnings: W001 final letter form in mid-word, W002 a keyword written backwards (a sign of
a visual-order file), W003 mixed Hebrew/ASCII identifiers on a line (`--warn-mixed`).

## Differences from the specification draft

- Header files use `.כ` (with `.ך` accepted), not `.גכ`.
- `[[הוצא]]` accepts ASCII identifier names only, because it is lowered to a pragma.
- The reserved prefix `גג` and `gml__` are enforced; the `_Pragma`-based export handles
  simple declarators only (not function-pointer declarators or struct bodies).

## Layout

```
src/keywords.rs   keyword and directive tables (single source of truth)
src/lexer.rs      logical-order lexer and spec checks
src/translate.rs  gimel to C and C to gimel
src/driver.rs     include-tree loading, work directory, compiler invocation
src/main.rs       the CLI
examples/         hello, fizzbuzz, multi-file (points + geometry), library, export
tests/cli.rs      end-to-end tests
```
