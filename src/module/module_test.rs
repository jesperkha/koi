use std::path::PathBuf;

use crate::module::ModulePath;

#[test]
fn test_modpath_is_stdlib() {
    let modpath = ModulePath::from("std.io");
    assert!(modpath.is_stdlib());

    let modpath2 = ModulePath::from("foo").to_std();
    assert!(modpath2.is_stdlib());
}

#[test]
fn test_modpath_is_library() {
    let modpath = ModulePath::from("lib.io");
    assert!(modpath.is_library());

    let modpath2 = ModulePath::from("foo").to_lib();
    assert!(modpath2.is_library());
}

#[test]
fn test_modpath_name() {
    let modpath = ModulePath::from("app.foo.bar");
    assert_eq!(modpath.name(), "bar");

    let modpath2 = ModulePath::from("main");
    assert_eq!(modpath2.name(), "main");
}

#[test]
fn test_modpath_first() {
    let modpath = ModulePath::from("app.foo.bar");
    assert_eq!(modpath.first(), "app");
}

#[test]
fn test_modpath_path_underscore() {
    let modpath = ModulePath::from("app.foo.bar");
    assert_eq!(modpath.path_underscore(), "app_foo_bar");
}

#[test]
fn test_modpath_from_pathbuf() {
    let modpath1 = ModulePath::from(&PathBuf::from("/lib/io/io.koi.h"));
    assert_eq!(modpath1.path(), "io");

    let modpath2 = ModulePath::from(&PathBuf::from("app/handler/handler.koi"));
    assert_eq!(modpath2.path(), "handler");
}
