use crate::module::ModulePath;

#[test]
fn test_modpath_is_stdlib() {
    let modpath = ModulePath::new_str("std.io");
    assert!(modpath.is_stdlib());

    let modpath2 = ModulePath::new_stdlib("foo");
    assert!(modpath2.is_stdlib());
}

#[test]
fn test_modpath_name() {
    let modpath = ModulePath::new_str("app.foo.bar");
    assert_eq!(modpath.name(), "bar");

    let modpath2 = ModulePath::new_str("main");
    assert_eq!(modpath2.name(), "main");
}

#[test]
fn test_modpath_first() {
    let modpath = ModulePath::new_str("app.foo.bar");
    assert_eq!(modpath.first(), "app");
}

#[test]
fn test_modpath_path_underscore() {
    let modpath = ModulePath::new_str("app.foo.bar");
    assert_eq!(modpath.path_underscore(), "app_foo_bar");
}
