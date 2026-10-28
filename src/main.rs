//! The `gimel` command line tool.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use gimel::diag::{self, Diag, SourceFile};
use gimel::driver::{self, BuildRequest, CcOptions, Failure, TreeOptions};
use gimel::keywords;
use gimel::mangle::Mangle;
use gimel::translate;

const USAGE: &str = "\
gimel (ג): C in Hebrew

USAGE:
    gimel <command> [options] [files]

COMMANDS:
    build <file.ג>...      Translate to C and compile with clang (or another C compiler)
    run <file.ג>... [-- args]
                           Build into a temporary directory and run the program
    emit-c <file.ג>        Print the generated C (or write a whole tree with --out-dir)
    check <file.ג>...      Translate and report diagnostics; produce nothing
    from-c <file.c>        Translate C source into gimel
    keywords               Print the keyword table
    help, version

FILES:
    .ג  source      .כ (or .ך)  header      ASCII aliases: .gml, .gmlh

OPTIONS:
    -o <path>              Output file (build, emit-c, from-c)
    -I <dir>               Add a directory searched for quoted includes (repeatable)
    -D <name[=value]>      Define a macro for the C compiler (repeatable)
    -O0 -O1 -O2 -O3 -Os    Optimization level, passed to the C compiler
    -g                     Debug info
    -c                     Compile to an object file, do not link
    --cc <cmd>             C compiler (default: $GIMEL_CC, then $CC, then clang, cc, gcc)
    --std <std>            C standard passed to the compiler (default: c2x)
    --keep <dir>           Keep the generated C in <dir>
    --out-dir <dir>        emit-c: write the generated C for the whole include tree
    --no-line              Omit #line directives in generated C
    --mangle <utf8|ascii>  How Hebrew identifiers reach the C compiler
                           (default: utf8, ascii on Windows)
    --warn-mixed           Warn when a line mixes Hebrew and ASCII identifiers
    -Werror                Treat gimel warnings as errors
    --isolate              Wrap quoted Hebrew in bidi isolates in diagnostics
                           (also GIMEL_ISOLATE=1)
    --format <fmt>         keywords: table (default), markdown, json, tsv
    --                     Everything after is passed to the C compiler
                           (for `run`: to the program)

EXAMPLES:
    gimel run examples/hello.ג
    gimel build examples/points.ג examples/geometry.ג -o points -O2
    gimel emit-c examples/hello.ג
    gimel keywords --format markdown
";

#[derive(Default)]
struct Args {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    include_dirs: Vec<PathBuf>,
    defines: Vec<String>,
    cc: Option<String>,
    std: Option<String>,
    opt: Option<String>,
    debug: bool,
    keep: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    mangle: Option<Mangle>,
    isolate: bool,
    warn_mixed: bool,
    werror: bool,
    compile_only: bool,
    no_line: bool,
    format: Option<String>,
    extra: Vec<String>,
    help: bool,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut i = 0;
    let value = |i: &mut usize, flag: &str| -> Result<String, String> {
        *i += 1;
        args.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
    };
    let mangle = |v: &str| Mangle::parse(v).ok_or_else(|| format!("unknown --mangle mode `{v}` (expected utf8 or ascii)"));
    while i < args.len() {
        let s = args[i].as_str();
        match s {
            "--" => {
                a.extra.extend(args[i + 1..].iter().cloned());
                break;
            }
            "-h" | "--help" => a.help = true,
            "-o" | "--output" => a.output = Some(value(&mut i, s)?.into()),
            "-I" => a.include_dirs.push(value(&mut i, s)?.into()),
            "-D" => a.defines.push(value(&mut i, s)?),
            "--cc" => a.cc = Some(value(&mut i, s)?),
            "--std" => a.std = Some(value(&mut i, s)?),
            "--keep" => a.keep = Some(value(&mut i, s)?.into()),
            "--out-dir" => a.out_dir = Some(value(&mut i, s)?.into()),
            "--format" => a.format = Some(value(&mut i, s)?),
            "--mangle" => a.mangle = Some(mangle(&value(&mut i, s)?)?),
            "-g" => a.debug = true,
            "-c" => a.compile_only = true,
            "--isolate" => a.isolate = true,
            "--warn-mixed" => a.warn_mixed = true,
            "-Werror" => a.werror = true,
            "--no-line" => a.no_line = true,
            _ if s.starts_with("--std=") => a.std = Some(s["--std=".len()..].into()),
            _ if s.starts_with("--cc=") => a.cc = Some(s["--cc=".len()..].into()),
            _ if s.starts_with("--keep=") => a.keep = Some(s["--keep=".len()..].into()),
            _ if s.starts_with("--out-dir=") => a.out_dir = Some(s["--out-dir=".len()..].into()),
            _ if s.starts_with("--output=") => a.output = Some(s["--output=".len()..].into()),
            _ if s.starts_with("--format=") => a.format = Some(s["--format=".len()..].into()),
            _ if s.starts_with("--mangle=") => a.mangle = Some(mangle(&s["--mangle=".len()..])?),
            _ if s.starts_with("-I") => a.include_dirs.push(s[2..].into()),
            _ if s.starts_with("-D") => a.defines.push(s[2..].into()),
            _ if s.starts_with("-O") => a.opt = Some(s.into()),
            _ if s.starts_with('-') && s.len() > 1 => return Err(format!("unknown option `{s}`")),
            _ => a.inputs.push(s.into()),
        }
        i += 1;
    }
    Ok(a)
}

fn tree_options(a: &Args) -> TreeOptions {
    TreeOptions {
        include_dirs: a.include_dirs.clone(),
        tr: translate::Options {
            mangle: a.mangle.unwrap_or_else(Mangle::platform_default),
            line_directive: !a.no_line,
            warn_mixed: a.warn_mixed,
            banner: true,
        },
    }
}

fn print_warnings(d: &[Diag]) {
    diag::print_all(d);
    eprintln!("gimel: {} warning{} emitted\n", d.len(), if d.len() == 1 { "" } else { "s" });
}

fn report(f: Failure) -> ExitCode {
    match f {
        Failure::Diags(d) => {
            diag::print_all(&d);
            let errors = d.iter().filter(|x| x.is_error()).count();
            let warnings = d.len() - errors;
            let w = if warnings > 0 { format!("; {warnings} warning{} emitted", if warnings == 1 { "" } else { "s" }) } else { String::new() };
            eprintln!("gimel: aborting due to {errors} error{}{w}", if errors == 1 { "" } else { "s" });
        }
        Failure::Msg(m) => eprintln!("gimel: error: {m}"),
    }
    ExitCode::from(1)
}

fn usage_error(msg: &str) -> ExitCode {
    eprintln!("gimel: {msg}\nTry `gimel help`.");
    ExitCode::from(2)
}

fn write_or_print(text: &str, out: Option<&Path>) -> Result<(), Failure> {
    match out {
        Some(p) => std::fs::write(p, text).map_err(|e| Failure::Msg(format!("cannot write {}: {e}", p.display()))),
        None => {
            use std::io::Write;
            std::io::stdout().write_all(text.as_bytes()).map_err(|e| Failure::Msg(format!("cannot write to stdout: {e}")))
        }
    }
}

fn cmd_build(a: &Args, run: bool) -> ExitCode {
    if a.inputs.is_empty() {
        return usage_error("no input files");
    }
    let req = BuildRequest {
        sources: a.inputs.clone(),
        output: a.output.clone(),
        keep: a.keep.clone(),
        tree: tree_options(a),
        cc: CcOptions {
            cc: a.cc.clone(),
            std: a.std.clone().unwrap_or_else(|| "c2x".into()),
            opt: a.opt.clone(),
            debug: a.debug,
            defines: a.defines.clone(),
            compile_only: a.compile_only,
            extra: if run { Vec::new() } else { a.extra.clone() },
        },
        deny_warnings: a.werror,
    };
    match driver::build(&req, run, &print_warnings) {
        Err(f) => report(f),
        Ok(built) => {
            if !run {
                return ExitCode::SUCCESS;
            }
            let status = Command::new(&built.output).args(&a.extra).status();
            built.cleanup();
            match status {
                Ok(s) => match s.code() {
                    Some(c) => ExitCode::from(c as u8),
                    None => ExitCode::from(1),
                },
                Err(e) => report(Failure::Msg(format!("cannot run the program: {e}"))),
            }
        }
    }
}

fn cmd_check(a: &Args) -> ExitCode {
    if a.inputs.is_empty() {
        return usage_error("no input files");
    }
    match driver::load_tree(&a.inputs, &tree_options(a)) {
        Err(f) => report(f),
        Ok(tree) => {
            if !tree.warnings.is_empty() {
                print_warnings(&tree.warnings);
                if a.werror {
                    return report(Failure::Msg("warnings treated as errors (-Werror)".into()));
                }
            }
            eprintln!("gimel: ok ({} file{})", tree.files.len(), if tree.files.len() == 1 { "" } else { "s" });
            ExitCode::SUCCESS
        }
    }
}

fn cmd_emit_c(a: &Args) -> ExitCode {
    if a.inputs.len() != 1 {
        return usage_error("emit-c takes exactly one input file");
    }
    let tree = match driver::load_tree(&a.inputs, &tree_options(a)) {
        Ok(t) => t,
        Err(f) => return report(f),
    };
    if !tree.warnings.is_empty() {
        print_warnings(&tree.warnings);
        if a.werror {
            return report(Failure::Msg("warnings treated as errors (-Werror)".into()));
        }
    }
    if let Some(dir) = &a.out_dir {
        return match driver::write_tree(&tree, dir, &a.include_dirs) {
            Ok(w) => {
                for f in &w.c_files {
                    println!("{}", f.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => report(Failure::Msg(format!("cannot write {}: {e}", dir.display()))),
        };
    }
    if tree.files.len() > 1 {
        eprintln!("gimel: note: this file includes gimel headers; use --out-dir to translate them too");
    }
    match write_or_print(&tree.files[0].output.text, a.output.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(f) => report(f),
    }
}

fn cmd_from_c(a: &Args) -> ExitCode {
    if a.inputs.len() != 1 {
        return usage_error("from-c takes exactly one input file");
    }
    let path = &a.inputs[0];
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return report(Failure::Msg(format!("cannot read {}: {e}", path.display()))),
    };
    match translate::from_c(&SourceFile::new(path.display().to_string(), text)) {
        Ok(out) => match write_or_print(&out.text, a.output.as_deref()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(f) => report(f),
        },
        Err(d) => report(Failure::Diags(d)),
    }
}

fn cmd_keywords(a: &Args) -> ExitCode {
    let text = match a.format.as_deref().unwrap_or("table") {
        "table" => keywords::render_table(),
        "markdown" | "md" => keywords::render_markdown(),
        "json" => keywords::render_json(),
        "tsv" => keywords::render_tsv(),
        other => return usage_error(&format!("unknown format `{other}` (expected table, markdown, json or tsv)")),
    };
    match write_or_print(&text, a.output.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(f) => report(f),
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args_os().skip(1).map(|a| a.to_string_lossy().into_owned()).collect();
    let Some(cmd) = argv.first().map(String::as_str) else {
        print!("{USAGE}");
        return ExitCode::from(2);
    };
    match cmd {
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        "version" | "-V" | "--version" => {
            println!("gimel {}", translate::VERSION);
            return ExitCode::SUCCESS;
        }
        _ => {}
    }
    let a = match parse(&argv[1..]) {
        Ok(a) => a,
        Err(e) => return usage_error(&e),
    };
    if a.help {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    diag::set_isolate(a.isolate || std::env::var_os("GIMEL_ISOLATE").map_or(false, |v| v == "1"));
    match cmd {
        "build" => cmd_build(&a, false),
        "run" => cmd_build(&a, true),
        "emit-c" => cmd_emit_c(&a),
        "check" => cmd_check(&a),
        "from-c" => cmd_from_c(&a),
        "keywords" => cmd_keywords(&a),
        other => usage_error(&format!("unknown command `{other}`")),
    }
}
