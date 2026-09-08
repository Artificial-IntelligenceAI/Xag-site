//! Runs every sample the site publishes through the real compiler.
//!
//!     cargo run --features prerender --bin verify-samples
//!     XAGC=/path/to/xagc cargo run --features prerender --bin verify-samples
//!
//! The brief says to run a sample before putting it on a page. Nothing said
//! anything about the day after. On 2026-09-08 the front page was found to be
//! showing `borrowing.xag`, which the compiler had refused since `ref` and
//! `refmut` became `loan` and `loanmut` — a program presented as Xag that Xag
//! rejects. It was found by accident, weeks late.
//!
//! This checks three things, because a sample can rot in three ways:
//!
//!   1. a program shown as working still compiles and runs,
//!   2. what it prints is still what the page says it prints,
//!   3. a program shown as refused is still refused, with the same error code.
//!
//! It is not in CI, because CI has no compiler. Somebody has to run it, and the
//! README says so.

use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_XAGC: &str = "/Users/ts/SafetyBolt language/build/xagc";

/// A program the site shows as working, and the output the page claims.
///
/// The claims are written here rather than read out of `content.rs` on purpose:
/// if this file simply echoed the page, it would agree with the page whatever
/// either of them said, which is not a check.
const RUNS: &[(&str, &str)] = &[
    ("samples/counting.xag", "sum to 10 = 55"),
    (
        "samples/borrowing.xag",
        "size: 5\nafter: hello!\nlonger: hello!\nkept: spare",
    ),
];

/// A program the site shows being refused, and the code it must be refused
/// with. The page prints the compiler's whole diagnostic, so the code is the
/// part worth pinning: the wording is allowed to improve.
const REFUSED: &[(&str, &str)] = &[("samples/moved.xag", "E0403")];

/// Samples that are in the repository but not on any page. They still have to
/// compile: a file that does not is a trap for whoever reaches for it next,
/// which is exactly how the stale one got published in the first place.
const UNPUBLISHED: &[&str] = &["samples/grouping.xag", "samples/holding.xag"];

fn main() {
    let xagc = PathBuf::from(std::env::var("XAGC").unwrap_or_else(|_| DEFAULT_XAGC.to_string()));
    if !xagc.is_file() {
        eprintln!(
            "no compiler at {}\nbuild it, or say where it is:\n    \
             XAGC=/path/to/xagc cargo run --features prerender --bin verify-samples",
            xagc.display()
        );
        std::process::exit(2);
    }

    let mut wrong = 0usize;
    let mut checked = 0usize;

    for (file, expected) in RUNS {
        checked += 1;
        match run(&xagc, file) {
            Ok(printed) => {
                let printed = printed.trim_end();
                if printed == *expected {
                    println!("  ok      {file}");
                } else {
                    wrong += 1;
                    println!("  DIFFERS {file}\n    page says: {expected:?}\n    prints:    {printed:?}");
                }
            }
            Err(why) => {
                wrong += 1;
                println!("  REFUSED {file}\n    shown as a working program, and the compiler says:\n    {why}");
            }
        }
    }

    for (file, code) in REFUSED {
        checked += 1;
        match run(&xagc, file) {
            Ok(printed) => {
                wrong += 1;
                println!("  RAN     {file}\n    shown as refused, but it ran and printed {printed:?}");
            }
            Err(why) => {
                if why.contains(code) {
                    println!("  ok      {file} (refused, {code})");
                } else {
                    wrong += 1;
                    println!("  DIFFERS {file}\n    page shows {code}, compiler says:\n    {why}");
                }
            }
        }
    }

    for file in UNPUBLISHED {
        checked += 1;
        match run(&xagc, file) {
            Ok(_) => println!("  ok      {file} (not published)"),
            Err(why) => {
                wrong += 1;
                println!("  REFUSED {file}\n    not published, but it should still compile:\n    {why}");
            }
        }
    }

    println!();
    if wrong == 0 {
        println!("{checked} samples, all still true");
    } else {
        println!("{checked} samples, {wrong} no longer true");
        println!("\nThe compiler is right and the page is wrong. Take the sample from the");
        println!("compiler's own examples/, re-run it, and correct whatever the page claims");
        println!("it prints.");
        std::process::exit(1);
    }
}

/// `Ok` with what it printed, or `Err` with the first line of the complaint.
fn run(xagc: &Path, file: &str) -> Result<String, String> {
    let out = match Command::new(xagc).arg("run").arg(file).output() {
        Ok(out) => out,
        Err(e) => return Err(format!("could not run the compiler: {e}")),
    };
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        // Diagnostics go to stderr. Reading only stdout found an empty string
        // and reported that the page was wrong about a sample that was fine.
        let said = String::from_utf8_lossy(&out.stderr);
        let line = said
            .lines()
            .find(|l| l.starts_with("Error code:"))
            .or_else(|| said.lines().find(|l| l.starts_with('`')))
            .unwrap_or("refused, and said nothing this could quote");
        let code = said
            .lines()
            .find(|l| l.starts_with("Error code:"))
            .unwrap_or("");
        Err(format!("{line} {code}").trim().to_string())
    }
}
