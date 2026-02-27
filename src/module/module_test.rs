use crate::{
    module::{ImportPath, ModulePath},
    util::FilePath,
};

#[test]
fn test_impath_is_x() {
    let impath = ImportPath::from("std.io");
    assert!(impath.is_stdlib());

    let impath = ImportPath::from("lib.io");
    assert!(impath.is_library());
}

#[test]
fn test_impath_name() {
    let impath = ImportPath::from("app.foo.bar");
    assert_eq!(impath.name(), "bar");

    let impath = ImportPath::from("main");
    assert_eq!(impath.name(), "main");
}

#[test]
fn test_impath_from_modpath() {
    let impath = ImportPath::from(&ModulePath::new("lib".into(), "mylib".into(), "foo".into()));
    assert_eq!(impath.path(), "lib.mylib.foo");
    assert_eq!(impath.name(), "foo");

    let impath = ImportPath::from(&ModulePath::new("".into(), "mylib".into(), "cmd".into()));
    assert_eq!(impath.path(), "cmd");
    assert_eq!(impath.name(), "cmd");
}

#[test]
fn test_modpath_underscore() {
    let modpath = ModulePath::new("".into(), "test".into(), "".into());
    assert_eq!(modpath.to_underscore(), "test");

    let modpath = ModulePath::new("lib".into(), "mylib".into(), "".into());
    assert_eq!(modpath.to_underscore(), "lib_mylib");

    let modpath = ModulePath::new("".into(), "".into(), "pkg.util".into());
    assert_eq!(modpath.to_underscore(), "pkg_util");

    let modpath = ModulePath::new("std".into(), "os".into(), "pkg.util".into());
    assert_eq!(modpath.to_underscore(), "std_os_pkg_util");
}

#[test]
fn test_modpath_is_x() {
    let modpath = ModulePath::new("lib".into(), "test".into(), "".into());
    assert!(modpath.is_library());

    let modpath = ModulePath::new("std".into(), "test".into(), "".into());
    assert!(modpath.is_stdlib());

    let modpath = ModulePath::new("".into(), "app".into(), "".into());
    assert!(!modpath.is_main());
    let modpath = modpath.to_main();
    assert!(modpath.is_main());
}

#[test]
fn test_modpath_from_filepath() {
    let modpath = ModulePath::from(&FilePath::from("/home/john/koi/external/mylib.util.koi.h"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "util");

    let modpath = ModulePath::from(&FilePath::from("mylib.koi.h"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "");
}

#[test]
fn test_modpath_from_impath() {
    let modpath = ModulePath::from(ImportPath::from("lib.mylib"));
    assert!(modpath.is_library());
    assert_eq!(modpath.prefix(), "lib");
    assert_eq!(modpath.package(), "mylib");
    assert_eq!(modpath.path(), "");

    let modpath = ModulePath::from(ImportPath::from("std.io.util"));
    assert!(modpath.is_stdlib());
    assert_eq!(modpath.prefix(), "std");
    assert_eq!(modpath.package(), "io");
    assert_eq!(modpath.path(), "util");

    let modpath = ModulePath::from(ImportPath::from("cmd"));
    assert_eq!(modpath.prefix(), "");
    assert_eq!(modpath.package(), "");
    assert_eq!(modpath.path(), "cmd");
}
