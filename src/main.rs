/// A wc clone built in Rust.

use std::process;

use ccwc::{run, Cli};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    println!("{cli:?}");

    match run(cli) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("Application error: {e}");
            process::exit(1);
        }
    };
}
