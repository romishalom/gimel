//! gimel (ג): C in Hebrew.
//!
//! gimel is C23 with Hebrew keywords written as commands and source stored in
//! logical order for right-to-left display. This crate translates gimel to C
//! (and C back to gimel) and drives a C compiler such as clang.
//!
//! ```
//! use gimel::{diag::SourceFile, translate};
//! let sf = SourceFile::new("hello.ג", "שלם התחל(ריק) { החזר 0; }");
//! let opts = translate::Options { line_directive: false, banner: false, ..Default::default() };
//! assert_eq!(translate::to_c(&sf, &opts).unwrap().text, "int main(void) { return 0; }");
//! ```

pub mod diag;
pub mod driver;
pub mod keywords;
pub mod lexer;
pub mod mangle;
pub mod stdlib;
pub mod translate;
