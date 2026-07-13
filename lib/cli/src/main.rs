use clap::Parser;
use ripsaw::cli::{Opts, cmd::cmd};

fn main() {
    std::process::exit(cmd(&Opts::parse(), ripsaw::stdlib::all()));
}
