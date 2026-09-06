//! Runs every program the playground ships through both the playground and the
//! real compiler, and complains if they disagree.
//!
//! The playground is a second implementation of a language, which is a thing
//! that drifts. This is the only reason to have any confidence in it, and it is
//! not automatic: somebody has to run it, with a built `xagc` to hand.
//!
//!     cargo run --features prerender --bin verify-play
//!     XAGC=/path/to/xagc cargo run --features prerender --bin verify-play
//!
//! A disagreement is not a bug in the playground to be patched into agreement.
//! It means the playground is telling readers something untrue, and the honest
//! fixes are to correct it or to stop claiming that much.

use std::path::{Path, PathBuf};
use std::process::Command;

use xag_site::play;

const DEFAULT_XAGC: &str = "/Users/ts/SafetyBolt language/build/xagc";

/// Every program the playground offers, and anything else worth pinning.
const PROGRAMS: &[(&str, &str)] = &[
    (
        "the example the playground opens with",
        include_str!("../samples/play/welcome.xag"),
    ),
    (
        "arithmetic and precedence",
        "START {\n\
         \x20   var.int64 'a' = [*2* + *3* x *4*];\n\
         \x20   var.int64 'b' = [*7* / *2*];\n\
         \x20   var.int64 'c' = [(*0* - *7*) / *2*];\n\
         \x20   var.int64 'd' = [(*0* - *7*) mod *3*];\n\
         \x20   print.stdout['a' str:*,* 'b' str:*,* 'c' str:*,* 'd' \\n];\n\
         }\n",
    ),
    (
        "a loop over both ends",
        "START {\n\
         \x20   var.mut.int64 'n' = [*0*];\n\
         \x20   loop.range.int64 'i' = [*1*, *10*] { set 'n' = ['n' + 'i']; }\n\
         \x20   print.stdout[str:*sum * 'n' \\n];\n\
         }\n",
    ),
    (
        "a name that is used after it changes",
        "const.int64 'step' = [*3*];\n\
         START {\n\
         \x20   var.mut.int64 'at' = [*0*];\n\
         \x20   loop.range.int64 'i' = [*1*, *4*] { set 'at' = ['at' + 'step' x 'i']; }\n\
         \x20   print.stdout[str:*at * 'at' \\n];\n\
         }\n",
    ),
    (
        "printing a written number, which says its type",
        "START { print.stdout[int64:*42* str:* and * int64:*7* \\n]; }\n",
    ),
    (
        "a sum printed straight, without a name to hold it",
        "START { var.int64 'a' = [*4*]; print.stdout[int64:*9* + *1* str:*,* 'a' x *3* \\n]; }\n",
    ),
    // Programs below are here to be refused. Agreeing to refuse is a much
    // weaker agreement than agreeing on an answer, but a playground that
    // accepts what `xagc` rejects teaches a syntax that does not exist.
    (
        "`mod` mixed with `+`, which has no settled order",
        "START { print.stdout[int64:*9* mod *5* + *1* \\n]; }\n",
    ),
    (
        "text between double quotes",
        "START { print.stdout[str:\"hello\" \\n]; }\n",
    ),
    (
        "a `const` inside `START`",
        "START { const.int64 'a' = [*1*]; print.stdout['a' \\n]; }\n",
    ),
    (
        "`set` on a name whose chain never said `mut`",
        "START { var.int64 'a' = [*1*]; set 'a' = [*2*]; print.stdout['a' \\n]; }\n",
    ),
    (
        "a name that was never declared",
        "START { print.stdout['nowhere' \\n]; }\n",
    ),
    (
        "dividing by nothing",
        "START { var.int64 'z' = [*1* / *0*]; print.stdout['z' \\n]; }\n",
    ),
];

fn main() {
    let xagc = std::env::var("XAGC").unwrap_or_else(|_| DEFAULT_XAGC.to_string());
    let xagc = PathBuf::from(xagc);
    if !xagc.is_file() {
        eprintln!(
            "verify-play: no compiler at {}. Build it, or set XAGC.",
            xagc.display()
        );
        std::process::exit(1);
    }

    let dir = std::env::temp_dir().join("xag-verify-play");
    let _ = std::fs::create_dir_all(&dir);

    let mut disagreed = 0usize;

    for (what, source) in PROGRAMS {
        let ours = play::run(source);
        let theirs = compiler_says(&xagc, &dir, source);

        match (&ours, &theirs) {
            (Ok(a), Ok(b)) if a == b => println!("  agree   {what}"),
            (Ok(a), Ok(b)) => {
                disagreed += 1;
                println!("  DIFFER  {what}\n    playground: {a:?}\n    xagc:       {b:?}");
            }
            (Err(a), Err(b)) => {
                // Both refuse it, which is the agreement that matters. The
                // wording is not expected to match and never will.
                println!("  agree   {what} (both refuse)\n    playground: {a}\n    xagc: {b}");
            }
            (Ok(a), Err(b)) => {
                disagreed += 1;
                println!("  DIFFER  {what}\n    playground ran it: {a:?}\n    xagc refused it:   {b}");
            }
            (Err(a), Ok(b)) => {
                disagreed += 1;
                println!("  DIFFER  {what}\n    playground refused it: {a}\n    xagc ran it:           {b:?}");
            }
        }
    }

    println!();
    if disagreed == 0 {
        println!("{} programs, no disagreements", PROGRAMS.len());
    } else {
        eprintln!(
            "{disagreed} of {} disagree. The playground is telling readers something untrue.",
            PROGRAMS.len()
        );
        std::process::exit(1);
    }
}

fn compiler_says(xagc: &Path, dir: &Path, source: &str) -> Result<String, String> {
    let file = dir.join("check.xag");
    std::fs::write(&file, source).map_err(|e| e.to_string())?;
    let out = Command::new(xagc)
        .arg("run")
        .arg(&file)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stdout)
            .lines()
            .find(|l| l.starts_with('`') || l.contains("Rule(s)"))
            .unwrap_or("refused")
            .to_string())
    }
}
