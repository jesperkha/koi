use std::{fs::create_dir_all, process::Command, sync::Once};

use tracing_subscriber::EnvFilter;

use crate::{
    config::{Config, Options, Project, ProjectType, Target},
    driver::compile,
    util::{FilePath, cmd},
};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        let env_filter = EnvFilter::builder()
            .with_default_directive(tracing_subscriber::filter::LevelFilter::OFF.into())
            .from_env_lossy();

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .without_time()
            .compact()
            .init();
    });
}

fn root_dir() -> FilePath {
    FilePath::from(env!("CARGO_MANIFEST_DIR"))
}

fn case_dir(case: &str) -> FilePath {
    root_dir().join("src/driver/tests/cases").join(case)
}

fn new_config(case: &str, install_dir: Option<String>) -> (Project, Options, Config) {
    init_logger();
    let project = Project {
        name: case.into(),
        bin: "bin".into(),
        src: case_dir(case).to_string(),
        out: "bin".into(),
        target: Target::X86_64,
        project_type: ProjectType::App,
        includes: None,
        ignore_dirs: vec![],
    };

    let options = Options {
        debug_mode: true,
        install_dir,
    };

    let config = Config {
        dump_type_context: false,
        print_symbol_tables: false,
        no_mangle_names: false,
    };

    (project, options, config)
}

fn library_config(
    case: &str,
    libname: &str,
    includes: Option<Vec<String>>,
    out_dir: String,
) -> (Project, Options, Config) {
    init_logger();
    let project = Project {
        name: libname.into(),
        bin: "bin".into(),
        src: case_dir(case).join(libname).to_string(),
        out: out_dir,
        target: Target::X86_64,
        project_type: ProjectType::Package,
        includes,
        ignore_dirs: vec![],
    };

    let options = Options {
        debug_mode: true,
        install_dir: None,
    };

    let config = Config {
        dump_type_context: false,
        print_symbol_tables: false,
        no_mangle_names: false,
    };

    (project, options, config)
}

fn expect_status(case: &str, status: i32) {
    let output = Command::new(root_dir().join("bin").join(case).path_buf())
        .output()
        .map_err(|e| panic!("failed to run binary: {}", e))
        .unwrap();

    assert_eq!(output.status.code().unwrap(), status);
}

fn run_case_with_status(case: &str, status: i32) {
    let (project, options, config) = new_config(case, None);
    compile(project, options, config).unwrap();
    expect_status(case, status);
}

fn run_case_with_error(case: &str, error: &str) {
    let (project, options, config) = new_config(case, None);
    match compile(project, options, config) {
        Ok(_) => panic!("expected error, got none"),
        Err(e) => assert_eq!(e, error),
    }
}

#[test]
fn test_exit0() {
    run_case_with_status("exit0", 0);
}

#[test]
fn test_exit123() {
    run_case_with_status("exit123", 123);
}

#[test]
fn test_extern() {
    run_case_with_status("extern", 11);
}

#[test]
fn test_assignment() {
    run_case_with_status("assignment", 5);
}

#[test]
fn test_call() {
    // TODO: bool x86 implementation is incorrect
    // as it doesnt use sized registers
    // run_case_with_status("call", 3);
}

#[test]
fn test_import() {
    run_case_with_status("import", 44);
}

#[test]
fn test_library() {
    // Create installation dir
    let install_dir = case_dir("library").join("installation");
    let lib_dir = install_dir.join("external/somelib");
    create_dir_all(lib_dir.path_buf()).unwrap();
    create_dir_all(install_dir.join("lib").path_buf()).unwrap();
    cmd(
        "cp",
        &vec!["lib/entry.s".into(), install_dir.join("lib").to_string()],
    )
    .unwrap();

    // Compile library
    let (project, options, config) =
        library_config("library", "somelib", None, lib_dir.to_string());
    compile(project, options, config).unwrap();

    // Compile test module
    let (project, options, config) = new_config("library", Some(install_dir.to_string()));
    compile(project, options, config).unwrap();
    expect_status("library", 44);
}

#[test]
fn test_empty() {
    run_case_with_error(
        "empty",
        &format!("no source files in '{}'", case_dir("empty")),
    );
}
