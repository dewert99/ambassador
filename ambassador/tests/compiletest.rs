extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.target_rustcflags = Some("-L ../target/debug/deps --edition 2018".to_owned());
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path

    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    compiletest::run_tests(&config);
}

#[test]
fn run_pass() {
    run_mode("run-pass");
}

#[test]
fn compile_fail() {
    run_mode("compile-fail");
}
