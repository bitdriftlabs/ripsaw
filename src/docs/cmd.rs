use crate::compiler::Function;
use clap::Parser;
use std::io;
use std::path::PathBuf;

use super::{build_functions_doc, document_functions_to_dir};

fn run(opts: &Opts, functions: &[Box<dyn Function>]) -> Result<(), io::Error> {
    if let Some(output) = &opts.output {
        document_functions_to_dir(functions, output, &opts.extension)
    } else {
        let built = build_functions_doc(functions);
        #[allow(clippy::print_stdout)]
        if opts.minify {
            println!(
                "{}",
                serde_json::to_string(&built).expect("FunctionDoc serialization should not fail")
            );
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&built)
                    .expect("FunctionDoc serialization should not fail")
            );
        }
        Ok(())
    }
}
