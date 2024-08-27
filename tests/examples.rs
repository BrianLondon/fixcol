use std::fs;
use std::path::Path;
use std::process::Command;

extern crate escargot;

fn read_expected_output_file(name: &str, variant: &str) -> String {
    let file_name = format!("{}.txt", variant);

    let cargo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = cargo.join("examples").join(name).join("expected_output").join(file_name);

    if path.exists() {
        fs::read_to_string(path).unwrap()
    } else {
        String::new()
    }
}

fn run_example_as_test(name: &str) {
    let expected_stdout = read_expected_output_file(name, "stdout");
    let expected_stderr = read_expected_output_file(name, "stderr");

    let example_bin = escargot::CargoBuild::new()
        .example(name)
        .run()
        .unwrap();

    let mut  cmd = Command::new(example_bin.path());
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();

    let output = cmd.output().unwrap();
    
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected_stdout);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected_stderr);
}

#[test]
fn test_habsburgs() {
    run_example_as_test("habsburgs");
}
