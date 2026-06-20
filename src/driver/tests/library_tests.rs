use std::fs::create_dir_all;

use crate::{
    config::{Codegen, Config, DriverPhase, Options, Project, ProjectType},
    driver::compile,
};

fn install_dir(name: &str) -> super::FilePath {
    super::case_dir("c_library").join("install").join(name)
}

/// Populate an installation directory with the directories and files the
/// driver expects: `include/koi.h`, an empty `lib/`, and an empty `external/`.
fn setup_install(dir: &super::FilePath) {
    create_dir_all(dir.join("include").path_buf()).unwrap();
    create_dir_all(dir.join("lib").path_buf()).unwrap();
    create_dir_all(dir.join("external").path_buf()).unwrap();
    std::fs::copy(
        super::installation_dir()
            .join("include")
            .join("koi.h")
            .path_buf(),
        dir.join("include").join("koi.h").path_buf(),
    )
    .unwrap();
}

/// Compile the mathlib package into `install_dir/external/mathlib/`.
/// `bin` is a test-unique subdirectory under the repo root used for
/// intermediate files, preventing parallel tests from clobbering each other.
fn compile_mathlib(dir: &super::FilePath, bin: &str) {
    let lib_out = dir.join("external").join("mathlib");
    create_dir_all(lib_out.path_buf()).unwrap();
    create_dir_all(super::root_dir().join(bin).path_buf()).unwrap();

    let (mut project, options, config) = super::library_config(
        "c_library",
        "mathlib",
        None,
        lib_out.to_string(),
        Codegen::C,
    );
    project.bin = bin.into();
    compile(project, options, config).unwrap();
}

fn app_project(test_name: &str, install_dir: &super::FilePath) -> (Project, Options, Config) {
    super::init_logger();
    let project = Project {
        name: test_name.into(),
        bin: "bin".into(),
        src: super::case_dir("c_library").join("app").to_string(),
        out: "bin".into(),
        project_type: ProjectType::App,
        includes: None,
        ignore_dirs: vec![],
        link_with: vec![],
    };
    let options = Options {
        debug_mode: true,
        install_dir: Some(install_dir.to_string()),
        codegen: Codegen::C,
    };
    let config = Config {
        driver_phase: DriverPhase::Full,
        dump_types: false,
        print_symbol_tables: false,
        no_mangle_names: false,
        comment_assembly: false,
    };
    (project, options, config)
}

/// Compile a package (library) in package mode and verify the archive and
/// header files are produced.
#[test]
fn test_package_compile() {
    let dir = install_dir("pkg");
    setup_install(&dir);
    compile_mathlib(&dir, "bin/lib_pkg");

    let lib_out = dir.join("external").join("mathlib");
    assert!(
        lib_out.join("libmathlib.a").path_buf().exists(),
        "expected libmathlib.a to be produced"
    );
    assert!(
        lib_out.join("mathlib.koi.h").path_buf().exists(),
        "expected mathlib.koi.h header to be produced"
    );
}

/// Compile an app that imports a separately-compiled library and verify the
/// compilation succeeds end-to-end (no panics, no error return).
#[test]
fn test_library_import() {
    let dir = install_dir("import");
    setup_install(&dir);
    compile_mathlib(&dir, "bin/lib_import");

    let (project, options, config) = app_project("c_lib_import", &dir);
    compile(project, options, config).unwrap();
}

/// Compile an app that calls library functions and verify the program
/// produces the expected result: add(3,4)=7, multiply(7,6)=42.
#[test]
fn test_library_usage() {
    let dir = install_dir("usage");
    setup_install(&dir);
    compile_mathlib(&dir, "bin/lib_usage");

    let (project, options, config) = app_project("c_lib_usage", &dir);
    compile(project, options, config).unwrap();
    super::expect_status("c_lib_usage", 42);
}
