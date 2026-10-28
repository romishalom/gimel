//! End-to-end tests of the `gimel` binary. Tests that need a C compiler skip
//! themselves when none is installed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn gimel() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gimel"))
}

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(name)
}

fn have_cc() -> bool {
    ["clang", "cc", "gcc"].iter().any(|c| {
        Command::new(c).arg("--version").output().map_or(false, |o| o.status.success())
    }) || std::env::var("GIMEL_CC").is_ok()
}

fn tempdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("gimel-test-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn version_and_help() {
    let o = gimel().arg("version").output().unwrap();
    assert!(o.status.success());
    assert!(stdout(&o).starts_with("gimel "));
    let o = gimel().arg("help").output().unwrap();
    assert!(stdout(&o).contains("USAGE"));
}

#[test]
fn keywords_table_formats() {
    for (fmt, needle) in [("table", "בדוק"), ("markdown", "| Control flow | `if` | **בדוק**"), ("json", "\"gimel\": \"בדוק\""), ("tsv", "Control flow\tif\tבדוק")] {
        let o = gimel().args(["keywords", "--format", fmt]).output().unwrap();
        assert!(o.status.success(), "{fmt}");
        assert!(stdout(&o).contains(needle), "{fmt}: {}", stdout(&o));
    }
}

#[test]
fn emit_c_prints_c() {
    let o = gimel().arg("emit-c").arg(example("hello.ג")).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    let c = stdout(&o);
    assert!(c.contains("int main(void) {"));
    assert!(c.contains("printf(\"שלום, עולם!\\n\")") || c.contains("הדפס(\"שלום, עולם!\\n\")"));
    assert!(c.contains("#include <stdio.h>"));
}

#[test]
fn emit_c_out_dir_translates_headers() {
    let dir = tempdir("outdir");
    let o = gimel().arg("emit-c").arg(example("points.ג")).arg("--out-dir").arg(&dir).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(dir.join("points.c").is_file());
    assert!(dir.join("geometry.h").is_file());
    let h = fs::read_to_string(dir.join("geometry.h")).unwrap();
    assert!(h.contains("struct נקודה"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_reports_errors_with_hints() {
    let dir = tempdir("check");
    let f = dir.join("bad.ג");
    fs::write(&f, "int x;\n#include <stdio.h>\n").unwrap();
    let o = gimel().arg("check").arg(&f).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    let e = stderr(&o);
    assert!(e.contains("G006") && e.contains("שלם"), "{e}");
    assert!(e.contains("G007") && e.contains("#כלול"), "{e}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_rejects_bidi_controls() {
    let dir = tempdir("bidi");
    let f = dir.join("evil.ג");
    fs::write(&f, "// \u{202E}\nשלם א;\n").unwrap();
    let o = gimel().arg("check").arg(&f).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("G001"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rejects_files_with_other_extensions() {
    let dir = tempdir("ext");
    let f = dir.join("x.c");
    fs::write(&f, "int main(void){return 0;}").unwrap();
    let o = gimel().arg("check").arg(&f).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("not a gimel file"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_header_is_reported() {
    let dir = tempdir("nohdr");
    let f = dir.join("a.ג");
    fs::write(&f, "#כלול \"אין.כ\"\n").unwrap();
    let o = gimel().arg("check").arg(&f).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("G013"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn werror_turns_warnings_into_failures() {
    let dir = tempdir("werror");
    let f = dir.join("w.ג");
    fs::write(&f, "שלם אםב;\n").unwrap();
    let o = gimel().arg("check").arg(&f).output().unwrap();
    assert!(o.status.success());
    assert!(stderr(&o).contains("W001"));
    let o = gimel().arg("check").arg("-Werror").arg(&f).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn from_c_translates_keywords() {
    let dir = tempdir("fromc");
    let f = dir.join("a.c");
    fs::write(&f, "#include <stdlib.h>\nint main(void) { return 0; }\n").unwrap();
    let o = gimel().arg("from-c").arg(&f).output().unwrap();
    assert!(o.status.success());
    assert_eq!(stdout(&o), "#כלול <כללי>\nשלם התחל(ריק) { החזר 0; }\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn run_hello() {
    if !have_cc() {
        return;
    }
    let o = gimel().arg("run").arg(example("hello.ג")).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "שלום, עולם!\n");
}

#[test]
fn run_forwards_exit_code_and_arguments() {
    if !have_cc() {
        return;
    }
    let dir = tempdir("exit");
    let f = dir.join("e.ג");
    fs::write(&f, "שלם התחל(שלם מספר, תו **ארגומנטים) { החזר מספר + 40; }\n").unwrap();
    let o = gimel().arg("run").arg(&f).args(["--", "a", "b"]).output().unwrap();
    assert_eq!(o.status.code(), Some(43), "{}", stderr(&o));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn multi_file_build_with_a_gimel_header() {
    if !have_cc() {
        return;
    }
    let dir = tempdir("multi");
    let exe = dir.join("points");
    let o = gimel().arg("build").arg(example("points.ג")).arg(example("geometry.ג")).arg("-o").arg(&exe).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(stdout(&run), "מרחק: 7\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn standard_library_names_work() {
    if !have_cc() {
        return;
    }
    let o = gimel().arg("run").arg(example("library.ג")).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "1 2 5 7 9 \nאורך: 5\n");
}

#[test]
fn ascii_mangling_builds_and_runs() {
    if !have_cc() {
        return;
    }
    let o = gimel().args(["run", "--mangle", "ascii"]).arg(example("library.ג")).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "1 2 5 7 9 \nאורך: 5\n");
}

#[test]
fn c_compiler_errors_point_at_the_gimel_file() {
    if !have_cc() {
        return;
    }
    let dir = tempdir("cerr");
    let f = dir.join("שגיאה.ג");
    fs::write(&f, "שלם התחל(ריק) {\n    החזר אין;\n}\n").unwrap();
    let o = gimel().arg("build").arg(&f).arg("-o").arg(dir.join("x")).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    let e = stderr(&o);
    assert!(e.contains("שגיאה.ג:2"), "{e}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn export_attribute_sets_the_symbol_name() {
    if !have_cc() {
        return;
    }
    let nm_ok = Command::new("nm").arg("--version").output().map_or(false, |o| o.status.success());
    if !nm_ok {
        return;
    }
    let dir = tempdir("export");
    let obj = dir.join("e.o");
    let o = gimel().args(["build", "-c"]).arg(example("export.ג")).arg("-o").arg(&obj).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    let nm = Command::new("nm").arg(&obj).output().unwrap();
    let syms = stdout(&nm);
    assert!(syms.contains("gimel_add") && syms.contains("gimel_counter"), "{syms}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn keep_directory_holds_generated_c() {
    if !have_cc() {
        return;
    }
    let dir = tempdir("keep");
    let o = gimel().arg("build").arg(example("hello.ג")).arg("-o").arg(dir.join("h")).arg("--keep").arg(dir.join("gen")).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(dir.join("gen").join("hello.c").is_file());
    let _ = fs::remove_dir_all(&dir);
}
