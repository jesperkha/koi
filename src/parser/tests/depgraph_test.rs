use crate::{
    ast::FileSet,
    parser::sort_by_dependency_graph,
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

fn get_ordered_files(files: &[TestFile]) -> Result<Vec<String>, String> {
    let parsed: Vec<FileSet> = files
        .iter()
        .map(|f| (&f.dep_name, must(parse_string(&f.src))))
        .map(|f| FileSet::new(f.0.clone(), vec![f.1]))
        .collect();

    let sorted = sort_by_dependency_graph(parsed)?;
    Ok(sorted.into_iter().map(|fs| fs.import_path).collect())
}

fn assert_correct_order(files: &[TestFile], order: &[&str]) {
    let ordered = get_ordered_files(files).unwrap_or_else(|e| panic!("expected no error: {}", e));
    assert_eq!(ordered.len(), order.len());
    for (i, f) in ordered.iter().enumerate() {
        assert_eq!(f, order[i]);
    }
}

fn assert_error(files: &[TestFile], msg: &str) {
    match get_ordered_files(files) {
        Ok(_) => panic!("expected error: '{}'", msg),
        Err(e) => assert_eq!(e, msg),
    }
}

#[test]
fn test_basic_import() {
    assert_correct_order(
        &vec![
            file(
                "third",
                r#"
                import second
            "#,
            ),
            file(
                "second",
                r#"
                import first
            "#,
            ),
            file(
                "first",
                r#"
            "#,
            ),
            file(
                "fourth",
                r#"
                import third
            "#,
            ),
        ],
        &vec!["first", "second", "third", "fourth"],
    );
}

#[test]
fn test_import_cycle() {
    assert_error(
        &vec![
            file(
                "first",
                r#"
                import second
            "#,
            ),
            file(
                "second",
                r#"
                import third
            "#,
            ),
            file(
                "third",
                r#"
                import first
            "#,
            ),
        ],
        "import cycle detected",
    );
}
