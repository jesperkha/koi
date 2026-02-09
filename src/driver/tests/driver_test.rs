use rexpect::spawn;

use crate::{
    config::{Config, load_config_file},
    driver::compile,
    util::cmd,
};

fn run(binary: &str) -> Result<String, String> {
    let mut p = spawn(binary, None).map_err(|e| format!("rexpect: {}", e))?;
    p.exp_eof().map_err(|e| format!("rexpect: {}", e))
}

fn compile_and_run_target(case: &str) -> Result<String, String> {
    let dir = format!("src/driver/tests/cases/{}", case);
    std::env::set_current_dir(&dir).map_err(|e| format!("cd {}: {}", dir, e))?;

    let (project, options, _) = load_config_file()?;
    let config = Config::test();

    compile(project, options, config)?;
    let res = run("./bin/main").map(|s| s.trim().into());
    cmd("rm", &vec!["-r".into(), "bin".into()])?;
    res
}

fn expect(case: &str, expect: &str) {
    match compile_and_run_target(case) {
        Ok(res) => assert_eq!(expect, res),
        Err(err) => panic!("expected no error, got: {}", err),
    }
}

fn expect_error(case: &str, error: &str) {
    match compile_and_run_target(case) {
        Ok(res) => panic!("expected error, got: {}", res),
        Err(err) => assert_eq!(error, err),
    }
}

#[test]
fn test_hello_world() {
    expect("hello_world", "Hello world!");
}
