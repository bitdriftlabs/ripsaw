extern crate lalrpop;

fn main() {
    println!("cargo:rerun-if-changed=src/parser/parser.lalrpop");
    lalrpop::Configuration::new()
        .always_use_colors()
        .emit_rerun_directives(true)
        .emit_whitespace(false)
        .process_dir("src/parser")
        .unwrap();
}
