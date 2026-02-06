use crate::{
    ast::{File, FileSet},
    config::Config,
    error::Diagnostics,
    module::ModulePath,
    parser::sort_by_dependency_graph,
    token::Source,
    typecheck::FileChecker,
    types::TypeContext,
    util::{must, new_source_map, new_source_map_from_files, parse_string},
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

fn check_files(files: &[TestFile]) -> Result<(), Diagnostics> {
    let parsed: Vec<FileSet> = files
        .iter()
        .map(|f| {
            let map = new_source_map(&f.src);
            (&f.dep_name, must(&map, parse_string(&f.src)))
        })
        .map(|f| {
            FileSet::new(
                ModulePath::new(f.0.clone()),
                vec![File::new(&Source::new_from_string(0, f.0), f.1)],
            )
        })
        .collect();

    let sorted = sort_by_dependency_graph(parsed).unwrap_or_else(|e| panic!("{}", e));

    let mut ctx = TypeContext::new();
    let config = Config::test();

    for fs in sorted {
        let checker = FileChecker::new(&mut ctx, &config);
        checker.check(fs)?;
    }

    Ok(())
}

fn assert_pass(files: &[TestFile]) {
    let map = new_source_map_from_files(&files.iter().map(|f| f.src.as_str()).collect::<Vec<_>>());
    must(&map, check_files(files))
}

fn assert_errors(files: &[TestFile], msgs: &[&str]) {
    match check_files(files) {
        Ok(_) => panic!("expected errors: {:?}", msgs),
        Err(errs) => {
            assert_eq!(
                errs.num_errors(),
                msgs.len(),
                "expected {} errors, got {}",
                msgs.len(),
                errs.num_errors()
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
        "could not resolve module path",
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
        "could not resolve module path",
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
        "already declared",
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
        "already declared",
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
        &vec!["already declared", "already declared"],
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
