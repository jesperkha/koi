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

fn installation_dir() -> FilePath {
    root_dir().join("src/driver/tests/installation")
}

fn case_dir(case: &str) -> FilePath {
    root_dir().join("src/driver/tests/cases").join(case)
}

fn new_config(case: &str) -> (Project, Options, Config) {
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
        link_with: vec![],
    };

    let options = Options {
        debug_mode: true,
        install_dir: Some(installation_dir().to_string()),
    };

    let config = Config {
        driver_phase: crate::config::DriverPhase::Full,
        dump_types: false,
        print_symbol_tables: false,
        no_mangle_names: false,
        comment_assembly: false,
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
        link_with: vec![],
    };

    let options = Options {
        debug_mode: true,
        install_dir: Some(installation_dir().to_string()),
    };

    let config = Config {
        driver_phase: crate::config::DriverPhase::Full,
        dump_types: false,
        print_symbol_tables: false,
        no_mangle_names: false,
        comment_assembly: false,
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
    let (project, options, config) = new_config(case);
    compile(project, options, config).unwrap();
    expect_status(case, status);
}

fn run_case_with_error(case: &str, error: &str) {
    let (project, options, config) = new_config(case);
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
    run_case_with_status("call", 3);
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
    let (project, mut options, config) = new_config("library");
    options.install_dir = Some(install_dir.to_string());
    compile(project, options, config).unwrap();
    expect_status("library", 44);
}

#[test]
fn test_excludes() {
    let (mut project, options, config) = new_config("excludes");
    project.ignore_dirs = vec!["excluded".into()];
    compile(project, options, config).unwrap();
    expect_status("excludes", 0);
}

#[test]
fn test_empty() {
    run_case_with_error(
        "empty",
        &format!("no source files in '{}'", case_dir("empty")),
    );
}

#[test]
fn test_duplicate_namespace() {
    run_case_with_status("duplicate_namespace", 0);
}

#[test]
fn test_param_alloc() {
    run_case_with_status("param_alloc", 1);
}

#[test]
fn test_binary_add() {
    run_case_with_status("binary_add", 5);
}

#[test]
fn test_binary_sub() {
    run_case_with_status("binary_sub", 7);
}

#[test]
fn test_binary_mul() {
    run_case_with_status("binary_mul", 12);
}

#[test]
fn test_binary_div() {
    run_case_with_status("binary_div", 5);
}

#[test]
fn test_unary_neg() {
    // negate(5) = -5, then -5 + 10 = 5
    run_case_with_status("unary_neg", 5);
}

#[test]
fn test_binary_compare() {
    run_case_with_status("binary_compare", 0);
}

#[test]
fn test_unary_not() {
    run_case_with_status("unary_not", 0);
}

// --- if / else ---

#[test]
fn test_if_taken() {
    // condition true, no else: body executes
    run_case_with_status("if_taken", 7);
}

#[test]
fn test_if_not_taken() {
    // condition false, no else: body skipped, n stays 0
    run_case_with_status("if_not_taken", 0);
}

#[test]
fn test_if_else_true_branch() {
    // condition true: if-body runs, else does NOT (tests jmp-past-else)
    run_case_with_status("if_else_true_branch", 3);
}

#[test]
fn test_if_else_false_branch() {
    // condition false: else-body runs
    run_case_with_status("if_else_false_branch", 5);
}

#[test]
fn test_if_elseif_first() {
    // first condition true: if-body runs, elseif+else skipped
    run_case_with_status("if_elseif_first", 10);
}

#[test]
fn test_if_elseif_middle() {
    // first condition false, elseif true: elseif-body runs, else skipped
    run_case_with_status("if_elseif_middle", 20);
}

#[test]
fn test_if_elseif_last() {
    // both conditions false: else-body runs
    run_case_with_status("if_elseif_last", 30);
}

#[test]
fn test_if_nested() {
    // outer true, inner false → inner else executes; outer else skipped
    run_case_with_status("if_nested", 4);
}

#[test]
fn test_if_computed() {
    // classify(3,5)=1, classify(9,4)=2, classify(6,6)=0 → sum=3
    run_case_with_status("if_computed", 3);
}

// --- while / break / continue ---

#[test]
fn test_while_count() {
    // count from 0 to 10, return final counter
    run_case_with_status("while_count", 10);
}

#[test]
fn test_while_zero_iters() {
    // condition false at entry: body never executes, n stays 42
    run_case_with_status("while_zero_iters", 42);
}

#[test]
fn test_while_break() {
    // infinite loop; break when i==5: exits with i==5
    run_case_with_status("while_break", 5);
}

#[test]
fn test_while_continue() {
    // 10 iterations, continue skips n++ when i==5: n==9
    run_case_with_status("while_continue", 9);
}

#[test]
fn test_while_nested_break() {
    // inner loop breaks at j==2; outer runs 3 times: n==6
    run_case_with_status("while_nested_break", 6);
}

#[test]
fn test_while_nested_continue() {
    // inner loop skips n++ when j==2; 3 outer × 4 inner = 12
    run_case_with_status("while_nested_continue", 12);
}
