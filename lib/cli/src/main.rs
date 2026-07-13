use clap::Parser;
use ripsaw::cli::{cmd::cmd, Opts};

fn main() {
    std::process::exit(cmd(&Opts::parse(), ripsaw::stdlib::all()));
}
