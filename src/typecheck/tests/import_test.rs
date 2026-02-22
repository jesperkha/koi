use crate::{
    ast::FileSet,
    config::Config,
    module::{ModuleGraph, ModulePath},
    parser::{parse_source_map, sort_by_dependency_graph},
    typecheck::check_filesets,
    types::TypeContext,
    util::{ErrorStream, must, new_source_map},
};

struct TestFile {
    dep_name: String,
    src: String,
}

fn file(name: &str, src: &str) -> TestFile {
    TestFile {
        dep_name: name.to_owned(),
        src: src.to_owned(),
    }
}

fn check_files(files: &[TestFile]) -> Result<(), ErrorStream> {
    let config = Config::test();
    let parsed: Vec<FileSet> = files
        .iter()
        .map(|f| {
            let map = new_source_map(&f.src);
            must(
                parse_source_map(ModulePath::new(f.dep_name.clone()), &map, &config)
                    .map_err(|e| ErrorStream::from(e)),
            )
        })
        .collect();

    let result = sort_by_dependency_graph(parsed).unwrap_or_else(|e| panic!("{}", e));
    let config = Config::test();
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    check_filesets(result.sets, &mut mg, &mut ctx, &config)?;

    Ok(())
}

fn assert_pass(files: &[TestFile]) {
    must(check_files(files))
}

fn assert_errors(files: &[TestFile], msgs: &[&str]) {
    match check_files(files) {
        Ok(_) => panic!("expected errors: {:?}", msgs),
        Err(errs) => {
            assert_eq!(
                errs.len(),
                msgs.len(),
                "expected {} errors, got {}",
                msgs.len(),
                errs.len()
            );
            for (i, &expected) in msgs.iter().enumerate() {
                assert_eq!(errs.get(i).message, expected);
            }
        }
    }
}

fn assert_error(files: &[TestFile], msg: &str) {
    assert_errors(files, &[msg]);
}

#[test]
fn test_basic_import() {
    assert_pass(&vec![
        file(
            "foo",
            r#"
            pub func doFoo() {}
        "#,
        ),
        file(
            "main",
            r#"
            import foo { doFoo }

            func main() int {
                doFoo()
                return 0
            }
        "#,
        ),
    ]);
}

#[test]
fn test_no_exported_symbol() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                pub func doFoo() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo { doBar }

                func main() int {
                    doBar()
                    return 0
                }
            "#,
            ),
        ],
        "module 'foo' has no export 'doBar'",
    );
}

#[test]
fn test_bad_import_path() {
    assert_error(
        &vec![file(
            "main",
            r#"
            import bar

            func main() int {
                return 0
            }
        "#,
        )],
        "could not resolve module import",
    );
    assert_error(
        &vec![
            file(
                "foo",
                r#"
            "#,
            ),
            file(
                "main",
                r#"
                import foo.bar

                func main() int {
                    return 0
                }
            "#,
            ),
        ],
        "could not resolve module import",
    );
}

#[test]
fn test_extern_export() {
    assert_pass(&vec![
        file(
            "foo",
            r#"
            pub extern func doFoo()
        "#,
        ),
        file(
            "main",
            r#"
            import foo { doFoo }

            func main() int {
                doFoo()
                return 0
            }
        "#,
        ),
    ]);
}

#[test]
fn test_many_imports() {
    assert_pass(&vec![
        file(
            "main",
            r#"
            import first { first }
            import second { second }
            import third { third }

            func main() int {
                first()
                second()
                third()
                return 0
            }
        "#,
        ),
        file(
            "third",
            r#"
            pub func third() {}
        "#,
        ),
        file(
            "second",
            r#"
            import third { third }

            pub func second() {
                third()
            }
        "#,
        ),
        file(
            "first",
            r#"
            import second { second }

            pub func first() {
                second()
            }
        "#,
        ),
    ]);
}

#[test]
fn test_namespace_import() {
    assert_pass(&vec![
        file(
            "foo",
            r#"
            pub func doFoo() {}
        "#,
        ),
        file(
            "main",
            r#"
            import foo

            func main() int {
                foo.doFoo()
                return 0
            }
        "#,
        ),
    ]);
}

#[test]
fn test_duplicate_symbol() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
            pub func doFoo() {}
        "#,
            ),
            file(
                "main",
                r#"
            import foo { doFoo }

            func doFoo() {}

            func main() int {
                foo.doFoo()
                return 0
            }
        "#,
            ),
        ],
        "already declared",
    );
}

#[test]
fn test_duplicate_symbol_2() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
            pub func doFoo() {}
        "#,
            ),
            file(
                "main",
                r#"
            import foo
            import foo { doFoo }

            func main() int {
                foo.doFoo()
                return 0
            }
        "#,
            ),
        ],
        "duplicate namespace 'foo'",
    );
}

#[test]
fn test_namespace_shadow_error() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                pub func doFoo() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo

                func main() int {
                    foo := 1
                    return 0
                }
            "#,
            ),
        ],
        "shadowing a namespace is not allowed",
    );
}

#[test]
fn test_namespace_as_expression_error() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                func f() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo

                func main() int {
                    a := foo
                    return 0
                }
            "#,
            ),
        ],
        "namespace cannot be used as a value",
    );
}

#[test]
fn test_import_alias() {
    assert_pass(&vec![
        file(
            "foo",
            r#"
                pub func f() {}
            "#,
        ),
        file(
            "main",
            r#"
                import foo as bar

                func main() int {
                    bar.f()
                    return 0
                }
            "#,
        ),
    ]);
    assert_pass(&vec![
        file(
            "foo",
            r#"
                pub func f() {}
            "#,
        ),
        file(
            "main",
            r#"
                import foo as bar
                import foo

                func main() int {
                    bar.f()
                    foo.f()
                    return 0
                }
            "#,
        ),
    ]);
}

#[test]
fn test_duplicate_alias() {
    assert_error(
        &vec![
            file(
                "bar",
                r#"
                func f() {}
            "#,
            ),
            file(
                "foo",
                r#"
                func f() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo as faz
                import bar as faz

                func main() int {
                    return 0
                }
            "#,
            ),
        ],
        "duplicate namespace 'faz'",
    );
}

#[test]
fn test_duplicate_explicit_imports() {
    assert_errors(
        &vec![
            file(
                "foo",
                r#"
                pub func a() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo { a }
                import foo { a }

                func main() int {
                    a()
                    return 0
                }
            "#,
            ),
        ],
        &vec!["duplicate namespace 'foo'", "already declared"],
    );
}

#[test]
fn test_duplicate_symbol_from_two_modules() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                pub func a() {}
            "#,
            ),
            file(
                "bar",
                r#"
                pub func a() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo { a }
                import bar { a }

                func main() int {
                    a()
                    return 0
                }
            "#,
            ),
        ],
        "already declared",
    );
}

#[test]
fn test_alias_shadowing_error() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                pub func f() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo as f

                func main() int {
                    f := 1
                    return 0
                }
            "#,
            ),
        ],
        "shadowing a namespace is not allowed",
    );
}

#[test]
fn test_import_private_symbol_error() {
    assert_error(
        &vec![
            file(
                "foo",
                r#"
                func private() {}
            "#,
            ),
            file(
                "main",
                r#"
                import foo { private }

                func main() int {
                    private()
                    return 0
                }
            "#,
            ),
        ],
        "module 'foo' has no export 'private'",
    );
}

#[test]
fn test_no_reexport() {
    assert_error(
        &vec![
            file(
                "one",
                r#"
                pub func f() {}
            "#,
            ),
            file(
                "two",
                r#"
                import one { f }
            "#,
            ),
            file(
                "main",
                r#"
                import two { f }

                func main() int {
                    return 0
                }
            "#,
            ),
        ],
        "module 'two' has no export 'f'",
    );
    assert_error(
        &vec![
            file(
                "one",
                r#"
                pub extern func f()
            "#,
            ),
            file(
                "two",
                r#"
                import one { f }
            "#,
            ),
            file(
                "main",
                r#"
                import two { f }

                func main() int {
                    return 0
                }
            "#,
            ),
        ],
        "module 'two' has no export 'f'",
    );
}
