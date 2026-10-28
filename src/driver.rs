//! Build driver.
//!
//! A build translates every gimel source file and every gimel header (`.כ`,
//! `.ך`) reachable through quoted `#כלול` directives, writes the generated C
//! into a work directory that mirrors the layout of the sources, and hands the
//! result to a C compiler (clang by default).

use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::diag::{quote, Diag, SourceFile};
use crate::translate::{self, Options, Output};

/// Why an operation failed.
#[derive(Debug)]
pub enum Failure {
    /// Source diagnostics (already structured, ready to print).
    Diags(Vec<Diag>),
    /// Anything else: IO errors, missing compiler, and so on.
    Msg(String),
}

impl From<String> for Failure {
    fn from(s: String) -> Self {
        Failure::Msg(s)
    }
}

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub include_dirs: Vec<PathBuf>,
    pub tr: Options,
}

#[derive(Debug)]
pub struct TreeFile {
    /// Canonical path of the gimel file.
    pub src: PathBuf,
    pub is_source: bool,
    pub output: Output,
}

#[derive(Debug)]
pub struct Tree {
    pub files: Vec<TreeFile>,
    /// Warnings from every file.
    pub warnings: Vec<Diag>,
}

fn read_utf8(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    String::from_utf8(bytes).map_err(|_| format!("{} is not valid UTF-8; gimel source must be UTF-8 in logical order", path.display()))
}

/// Translate `sources` and everything they include.
pub fn load_tree(sources: &[PathBuf], opts: &TreeOptions) -> Result<Tree, Failure> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<(PathBuf, String, bool)> = VecDeque::new();
    for s in sources {
        let shown = s.display().to_string();
        if !translate::is_source_path(&shown) && !translate::is_header_path(&shown) {
            return Err(Failure::Msg(format!(
                "{shown}: not a gimel file (expected a .ג source, a .כ header, or the ASCII aliases .gml and .gmlh)"
            )));
        }
        let canon = s.canonicalize().map_err(|e| Failure::Msg(format!("cannot open {shown}: {e}")))?;
        if seen.insert(canon.clone()) {
            queue.push_back((canon, shown.clone(), !translate::is_header_path(&shown)));
        }
    }
    let mut files = Vec::new();
    let mut errors: Vec<Diag> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();
    while let Some((canon, shown, is_source)) = queue.pop_front() {
        let text = read_utf8(&canon)?;
        let sf = SourceFile::new(shown.clone(), text);
        match translate::to_c(&sf, &opts.tr) {
            Err(d) => {
                for x in d {
                    if x.is_error() {
                        errors.push(x);
                    } else {
                        warnings.push(x);
                    }
                }
            }
            Ok(out) => {
                warnings.extend(out.diags.iter().cloned());
                let base = canon.parent().map(Path::to_path_buf).unwrap_or_default();
                for inc in out.includes.iter().filter(|i| i.quoted && translate::is_header_path(&i.path)) {
                    let mut cands = vec![(base.join(&inc.path), Path::new(&shown).parent().map(|p| p.join(&inc.path)))];
                    for d in &opts.include_dirs {
                        cands.push((d.join(&inc.path), Some(d.join(&inc.path))));
                    }
                    match cands.into_iter().find(|(p, _)| p.is_file()) {
                        Some((p, disp)) => {
                            let c = p.canonicalize().map_err(|e| Failure::Msg(format!("cannot open {}: {e}", p.display())))?;
                            if seen.insert(c.clone()) {
                                let disp = disp.unwrap_or_else(|| p.clone()).display().to_string();
                                queue.push_back((c, disp, false));
                            }
                        }
                        None => errors.push(
                            sf.error("G013", inc.line, inc.col, inc.path.chars().count() + 2, format!("cannot find gimel header {}", quote(&inc.path)))
                                .with_hint("quoted includes are searched next to the including file, then in each -I directory"),
                        ),
                    }
                }
                files.push(TreeFile { src: canon, is_source, output: out });
            }
        }
    }
    if !errors.is_empty() {
        errors.extend(warnings);
        return Err(Failure::Diags(errors));
    }
    Ok(Tree { files, warnings })
}

/// Result of [`write_tree`].
#[derive(Debug)]
pub struct Written {
    /// Generated `.c` files, one per gimel source.
    pub c_files: Vec<PathBuf>,
    /// Include directories for the C compiler, in search order.
    pub include_dirs: Vec<PathBuf>,
}

fn common_dir(paths: &[&Path]) -> PathBuf {
    let parts = |p: &Path| -> Vec<OsString> { p.parent().unwrap_or(p).components().map(|c| c.as_os_str().to_os_string()).collect() };
    let mut it = paths.iter();
    let mut common = match it.next() {
        Some(p) => parts(p),
        None => return PathBuf::new(),
    };
    for p in it {
        let other = parts(p);
        let n = common.iter().zip(other.iter()).take_while(|(a, b)| a == b).count();
        common.truncate(n);
    }
    let mut out = PathBuf::new();
    for c in common {
        out.push(c);
    }
    out
}

/// Write the generated C into `out_dir`, mirroring the layout of the sources so
/// that relative `#include "…"` paths keep working.
pub fn write_tree(tree: &Tree, out_dir: &Path, user_includes: &[PathBuf]) -> io::Result<Written> {
    let srcs: Vec<&Path> = tree.files.iter().map(|f| f.src.as_path()).collect();
    let root = common_dir(&srcs);
    let mut c_files = Vec::new();
    let mut include_dirs = Vec::new();
    for f in &tree.files {
        let rel = f.src.strip_prefix(&root).unwrap_or(&f.src);
        let ext = if f.is_source { "c" } else { "h" };
        let dest = out_dir.join(rel).with_extension(ext);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, &f.output.text)?;
        if f.is_source {
            c_files.push(dest);
        }
    }
    for d in user_includes {
        if let Ok(canon) = d.canonicalize() {
            if let Ok(rel) = canon.strip_prefix(&root) {
                include_dirs.push(out_dir.join(rel));
            }
        }
        include_dirs.push(d.clone());
    }
    // Plain C headers that sit next to gimel files stay in the source tree.
    for f in &tree.files {
        if let Some(p) = f.src.parent() {
            if !include_dirs.iter().any(|d| d == p) {
                include_dirs.push(p.to_path_buf());
            }
        }
    }
    Ok(Written { c_files, include_dirs })
}

#[derive(Debug, Clone)]
pub struct CcOptions {
    /// Compiler command; may contain arguments (for example `zig cc`).
    pub cc: Option<String>,
    pub std: String,
    pub opt: Option<String>,
    pub debug: bool,
    pub defines: Vec<String>,
    pub compile_only: bool,
    /// Extra arguments passed through verbatim (everything after `--`).
    pub extra: Vec<String>,
}

impl Default for CcOptions {
    fn default() -> Self {
        CcOptions { cc: None, std: "c2x".into(), opt: None, debug: false, defines: Vec::new(), compile_only: false, extra: Vec::new() }
    }
}

fn works(cmd: &str) -> bool {
    let mut parts = cmd.split_whitespace();
    let Some(prog) = parts.next() else { return false };
    Command::new(prog)
        .args(parts)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}

/// Pick the C compiler: `--cc`, then `GIMEL_CC`, then `CC`, then clang, cc, gcc.
pub fn find_cc(explicit: Option<&str>) -> Result<String, String> {
    if let Some(c) = explicit {
        return Ok(c.to_string());
    }
    for var in ["GIMEL_CC", "CC"] {
        if let Ok(c) = std::env::var(var) {
            if !c.trim().is_empty() {
                return Ok(c);
            }
        }
    }
    for c in ["clang", "cc", "gcc"] {
        if works(c) {
            return Ok(c.to_string());
        }
    }
    Err("no C compiler found: install clang, or pass --cc / set GIMEL_CC".into())
}

pub fn run_cc(cc: &str, opts: &CcOptions, written: &Written, output: &Path) -> Result<ExitStatus, String> {
    let mut parts = cc.split_whitespace();
    let prog = parts.next().ok_or("empty compiler command")?;
    let mut cmd = Command::new(prog);
    cmd.args(parts);
    cmd.arg(format!("-std={}", opts.std));
    if let Some(o) = &opts.opt {
        cmd.arg(o);
    }
    if opts.debug {
        cmd.arg("-g");
    }
    if opts.compile_only {
        cmd.arg("-c");
    }
    for d in &opts.defines {
        cmd.arg(format!("-D{d}"));
    }
    for d in &written.include_dirs {
        cmd.arg("-I").arg(d);
    }
    cmd.arg("-o").arg(output);
    cmd.args(&written.c_files);
    cmd.args(&opts.extra);
    cmd.status().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            format!("cannot run the C compiler `{prog}`: not found; install clang, or pass --cc / set GIMEL_CC")
        } else {
            format!("cannot run the C compiler `{prog}`: {e}")
        }
    })
}

fn unique_work_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!("gimel-{}-{nanos}", std::process::id()))
}

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub sources: Vec<PathBuf>,
    pub output: Option<PathBuf>,
    /// Keep the generated C in this directory instead of a temporary one.
    pub keep: Option<PathBuf>,
    pub tree: TreeOptions,
    pub cc: CcOptions,
    /// Fail when the translator produced warnings (`-Werror`).
    pub deny_warnings: bool,
}

#[derive(Debug)]
pub struct Built {
    pub output: PathBuf,
    pub warnings: Vec<Diag>,
    /// Temporary work directory to delete when done (None when `keep` was used).
    pub temp: Option<PathBuf>,
}

impl Built {
    pub fn cleanup(&self) {
        if let Some(t) = &self.temp {
            let _ = fs::remove_dir_all(t);
        }
    }
}

/// Default output name: the stem of the first source, `.o` for `-c`.
pub fn default_output(first: &Path, compile_only: bool) -> PathBuf {
    let stem = first.file_stem().map(|s| s.to_os_string()).unwrap_or_else(|| "a".into());
    let mut p = PathBuf::from(stem);
    if compile_only {
        p.set_extension("o");
    } else if cfg!(windows) {
        p.set_extension("exe");
    }
    p
}

/// Translate, write and compile. `into_work_dir` puts the product in the work
/// directory (used by `gimel run`). `on_warnings` is called with the
/// translator's warnings before the C compiler starts.
pub fn build(req: &BuildRequest, into_work_dir: bool, on_warnings: &dyn Fn(&[Diag])) -> Result<Built, Failure> {
    let tree = load_tree(&req.sources, &req.tree)?;
    if !tree.warnings.is_empty() {
        on_warnings(&tree.warnings);
        if req.deny_warnings {
            return Err(Failure::Msg("warnings treated as errors (-Werror)".into()));
        }
    }
    let (work, temp) = match &req.keep {
        Some(k) => (k.clone(), None),
        None => {
            let w = unique_work_dir();
            (w.clone(), Some(w))
        }
    };
    let result = (|| -> Result<PathBuf, Failure> {
        let written = write_tree(&tree, &work, &req.tree.include_dirs).map_err(|e| Failure::Msg(format!("cannot write {}: {e}", work.display())))?;
        let cc = find_cc(req.cc.cc.as_deref())?;
        let output = if into_work_dir {
            work.join(if cfg!(windows) { "program.exe" } else { "program" })
        } else {
            req.output.clone().unwrap_or_else(|| default_output(&req.sources[0], req.cc.compile_only))
        };
        let status = run_cc(&cc, &req.cc, &written, &output)?;
        if !status.success() {
            let kept = if req.keep.is_some() { format!("; generated C is in {}", work.display()) } else { String::new() };
            return Err(Failure::Msg(format!("the C compiler `{cc}` failed ({status}){kept}")));
        }
        Ok(output)
    })();
    match result {
        Ok(output) => Ok(Built { output, warnings: tree.warnings, temp }),
        Err(e) => {
            if let Some(t) = &temp {
                let _ = fs::remove_dir_all(t);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_dir_of_siblings_and_cousins() {
        let a = Path::new("/p/src/a.ג");
        let b = Path::new("/p/inc/b.כ");
        assert_eq!(common_dir(&[a, b]), PathBuf::from("/p"));
        assert_eq!(common_dir(&[a]), PathBuf::from("/p/src"));
    }

    #[test]
    fn default_output_names() {
        assert_eq!(default_output(Path::new("x/שלום.ג"), true), PathBuf::from("שלום.o"));
        if !cfg!(windows) {
            assert_eq!(default_output(Path::new("x/שלום.ג"), false), PathBuf::from("שלום"));
        }
    }
}
