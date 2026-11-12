//! End-to-end suite: drives the compiled `kernelino` binary through stdin and
//! compares its whole stdout with the golden files of `tests/e2e/`.
//!
//! Each scenario is a pair `tests/e2e/<name>.in` (what is typed on stdin) and
//! `tests/e2e/<name>.out` (the expected stdout, ANSI sequences stripped).
//! Run with `KERNELINO_BLESS=1` to write the actual outputs to the `.out` files.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Removes ANSI CSI sequences (`ESC [` params final-byte), e.g. the `\x1b[2J\x1b[1;1H`
/// written by `utils::clear_terminal` at startup and by `clear`.
fn strip_ansi(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            // Parameter and intermediate bytes, then one final byte in 0x40..=0x7E.
            for c in chars.by_ref() {
                if ('\x40'..='\x7e').contains(&c) {
                    break;
                }
            }
        } else {
            stripped.push(c);
        }
    }
    stripped
}

/// Spawns the binary, writes `input` to stdin, closes it, waits.
/// Panics if the process does not exit successfully or writes to stderr.
fn run_kernelino(input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kernelino"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Dropping stdin closes it, so the shell sees the end of input.
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}\n{stderr}", output.status);
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    strip_ansi(&String::from_utf8(output.stdout).unwrap())
}

fn scenarios_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
}

#[test]
fn scenarios() {
    let bless = std::env::var("KERNELINO_BLESS").as_deref() == Ok("1");

    let mut inputs: Vec<PathBuf> = fs::read_dir(scenarios_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "in"))
        .collect();
    inputs.sort();
    assert!(!inputs.is_empty(), "no scenario in {:?}", scenarios_dir());

    let mut failures = Vec::new();
    for input_path in &inputs {
        let name = input_path.file_stem().unwrap().to_string_lossy();
        let output_path = input_path.with_extension("out");
        let actual = run_kernelino(&fs::read_to_string(input_path).unwrap());

        if bless {
            fs::write(&output_path, &actual).unwrap();
            continue;
        }
        match fs::read_to_string(&output_path) {
            Ok(expected) if expected == actual => {}
            Ok(expected) => failures.push(format!(
                "scenario `{name}`\n--- expected\n{expected}\n--- actual\n{actual}"
            )),
            Err(err) => failures.push(format!("scenario `{name}`: {output_path:?}: {err}")),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} scenarios failed (bless with KERNELINO_BLESS=1):\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n\n")
    );
}
