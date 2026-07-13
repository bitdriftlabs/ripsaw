use std::io::Write;
use std::process::{Command, Stdio};

fn run_ripsaw_repl(input: Option<&str>, args: &[&str]) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ripsaw"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ripsaw process");

    let mut stdin = child
        .stdin
        .take()
        .expect("failed to take stdin for child ripsaw cli");

    if let Some(input) = input {
        stdin
            .write_all(format!("{input}\n").as_bytes())
            .expect("failed to write input to stdin");
    }

    // Send "exit" to close the REPL
    stdin
        .write_all(b"exit\n")
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait on child");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
// abs is just a random stdlib function
fn test_abs_works() {
    let stdout = run_ripsaw_repl(Some("abs(-1)"), &["-q"]);
    assert_eq!(stdout, "1\n\n");
}
