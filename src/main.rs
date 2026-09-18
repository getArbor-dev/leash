//! Budgeted context for coding agents. Out-of-radius patches don't land.

use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    // Drain stdin only when the command reads it; `run` owns that.
    let code = leash::app::run(&args, &mut stdin, &mut stdout, &mut stderr);
    let _ = stdout.flush();
    let _ = stderr.flush();
    let _ = stdin;
    std::process::exit(code);
}
