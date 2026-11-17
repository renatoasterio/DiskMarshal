//! Project automation, run as `cargo xtask <task>`.
//!
//! `check` is the quality gate: the same steps run locally and in CI, and a
//! change is not done while any of them fails.

#![forbid(unsafe_code)]

use std::env;
use std::process::{Command, ExitCode};

/// The quality gate, in the order it runs: name and `cargo` arguments.
const CHECK_STEPS: [(&str, &[&str]); 4] = [
    ("format", &["fmt", "--all", "--", "--check"]),
    (
        "lint",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ),
    ("test", &["test", "--workspace"]),
    ("audit", &["deny", "check"]),
];

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("check") => check(),
        _ => {
            eprintln!("usage: cargo xtask check");
            ExitCode::from(2)
        }
    }
}

fn check() -> ExitCode {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    for (name, arguments) in CHECK_STEPS {
        eprintln!("==> {name}: cargo {}", arguments.join(" "));
        let succeeded = Command::new(&cargo)
            .args(arguments)
            .status()
            .is_ok_and(|status| status.success());
        if !succeeded {
            eprintln!("xtask: step `{name}` failed");
            return ExitCode::FAILURE;
        }
    }
    eprintln!("xtask: all checks passed");
    ExitCode::SUCCESS
}
