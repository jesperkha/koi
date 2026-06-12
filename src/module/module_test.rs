use crate::{
    common::{FilePath, must, parse_string},
    module::{ImportPath, ModulePath},
};

#[test]
fn test_impath_is_stdlib() {
    let impath = ImportPath::from("std.io");
    assert!(impath.is_stdlib());
}

#[test]
fn test_impath_is_library() {
    let impath = ImportPath::from("lib.io");
    assert!(impath.is_library());
}

#[test]
fn test_impath_name_multi_segment() {
    let impath = ImportPath::from("app.foo.bar");
    assert_eq!(impath.name(), "bar");
}

#[test]
fn test_impath_name_single_segment() {
    let impath = ImportPath::from("main");
    assert_eq!(impath.name(), "main");
}

#[test]
fn test_impath_from_modpath_with_prefix() {
    let impath = ImportPath::from(&ModulePath::new("lib".into(), "mylib".into(), "foo".into()));
    assert_eq!(impath.path(), "lib.mylib.foo");
    assert_eq!(impath.name(), "foo");
}

#[test]
fn test_impath_from_modpath_no_prefix() {
    let impath = ImportPath::from(&ModulePath::new("".into(), "mylib".into(), "cmd".into()));
    assert_eq!(impath.path(), "cmd");
    assert_eq!(impath.name(), "cmd");
}

#[test]
fn test_modpath_underscore_package_only() {
    let modpath = ModulePath::new("".into(), "test".into(), "".into());
    assert_eq!(modpath.to_underscore(), "test");
}

#[test]
fn test_modpath_underscore_prefix_and_package() {
    let modpath = ModulePath::new("lib".into(), "mylib".into(), "".into());
    assert_eq!(modpath.to_underscore(), "lib_mylib");
}

#[test]
fn test_modpath_underscore_path_only() {
    let modpath = ModulePath::new("".into(), "".into(), "pkg.util".into());
    assert_eq!(modpath.to_underscore(), "pkg_util");
}

#[test]
fn test_modpath_underscore_all_parts() {
    let modpath = ModulePath::new("std".into(), "os".into(), "pkg.util".into());
    assert_eq!(modpath.to_underscore(), "std_os_pkg_util");
}

#[test]
fn test_modpath_is_library() {
    let modpath = ModulePath::new("lib".into(), "test".into(), "".into());
    assert!(modpath.is_library());
}

#[test]
fn test_modpath_is_main() {
    let modpath = ModulePath::new("".into(), "app".into(), "".into());
    assert!(!modpath.is_main());
    let modpath = modpath.to_main();
    assert!(modpath.is_main());
}

#[test]
fn test_modpath_from_filepath_with_path() {
    let modpath = ModulePath::from(&FilePath::from("/home/john/koi/external/mylib.util.koi.h"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "util");
}

#[test]
fn test_modpath_from_filepath_package_only() {
    let modpath = ModulePath::from(&FilePath::from("mylib.koi.h"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "");
}

#[test]
fn test_modpath_from_impath_with_prefix() {
    let modpath = ModulePath::from(ImportPath::from("lib.mylib"));
    assert!(modpath.is_library());
    assert_eq!(modpath.prefix(), "lib");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "");
}

#[test]
fn test_modpath_from_impath_multi_segment() {
    let modpath = ModulePath::from(ImportPath::from("std.io.util"));
    assert_eq!(modpath.prefix(), "std");
    assert_eq!(modpath.package(), "io");
    assert_eq!(modpath.path(), "util");
}

#[test]
fn test_modpath_from_impath_path_only() {
    let modpath = ModulePath::from(ImportPath::from("cmd"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "");
    assert_eq!(modpath.path(), "cmd");
}

#[test]
fn test_std_import_rewritten_to_lib_std_single_segment() {
    // `std.foo` written by the user must be normalised to `lib.std.foo`
    let ast = must(parse_string("import std.io"));
    let impath = ImportPath::from(&ast.imports[0]);
    assert_eq!(impath.path(), "lib.std.io");
    assert!(impath.is_library());
    assert!(!impath.is_stdlib());
}

#[test]
fn test_std_import_rewritten_to_lib_std_multi_segment() {
    let ast = must(parse_string("import std.collections.list"));
    let impath = ImportPath::from(&ast.imports[0]);
    assert_eq!(impath.path(), "lib.std.collections.list");
}

#[test]
fn test_non_std_lib_import_unchanged() {
    let ast = must(parse_string("import lib.mylib.util"));
    let impath = ImportPath::from(&ast.imports[0]);
    assert_eq!(impath.path(), "lib.mylib.util");
}

#[test]
fn test_non_std_app_import_unchanged() {
    let ast = must(parse_string("import myapp.utils"));
    let impath = ImportPath::from(&ast.imports[0]);
    assert_eq!(impath.path(), "myapp.utils");
}
