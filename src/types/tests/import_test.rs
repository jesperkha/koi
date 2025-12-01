use crate::{
    ast::FileSet,
    config::Config,
    error::ErrorSet,
    parser::sort_by_dependency_graph,
    types::{DepMap, type_check},
    util::{must, parse_string},
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

fn check_files(files: &[TestFile]) -> Result<(), ErrorSet> {
    let parsed: Vec<FileSet> = files
        .iter()
        .map(|f| (&f.dep_name, must(parse_string(&f.src))))
        .map(|f| FileSet::new(f.0.clone(), vec![f.1]))
        .collect();

    let sorted = sort_by_dependency_graph(parsed).unwrap_or_else(|e| panic!("{}", e));

    let mut deps = DepMap::empty();
    let config = Config::test();
    for fs in sorted {
        let _ = type_check(fs, &mut deps, &config)?;
    }

    Ok(())
}

fn assert_pass(files: &[TestFile]) {
    must(check_files(files))
}

fn assert_error(files: &[TestFile], msg: &str) {
    match check_files(files) {
        Ok(_) => panic!("expected error: '{}'", msg),
        Err(errs) => {
            assert!(errs.len() == 1, "expected one error, got {}", errs.len());
            assert_eq!(errs.get(0).message, msg);
        }
    }
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
        "package 'foo' has no export 'doBar'",
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
        "dependency not found",
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

// #[test]
// fn test_namespace_import() {
//     assert_pass(&vec![
//         file(
//             "foo",
//             r#"
//             pub func doFoo() {}
//         "#,
//         ),
//         file(
//             "main",
//             r#"
//             import foo

//             func main() int {
//                 foo.doFoo()
//                 return 0
//             }
//         "#,
//         ),
//     ]);
// }
