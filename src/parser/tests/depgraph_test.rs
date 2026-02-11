use crate::{
    ast::FileSet,
    config::Config,
    module::ModulePath,
    parser::{parser::source_map_to_fileset, sort_by_dependency_graph},
    util::new_source_map,
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
    let p = files
        .iter()
        .map(|f| {
            let map = new_source_map(&f.src);
            let config = Config::test();
            source_map_to_fileset(ModulePath::new_str(&f.dep_name), &map, &config)
                .map_err(|e| e.render(&map))
        })
        .collect::<Vec<_>>();

    let parsed: Result<Vec<FileSet>, String> = p.into_iter().collect();

    let sorted = sort_by_dependency_graph(parsed?)?;
    Ok(sorted
        .into_iter()
        .map(|fs| fs.modpath.path().to_owned())
        .collect())
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

fn assert_unordered_eq(actual: Vec<String>, expected: &[&str]) {
    let mut a: Vec<String> = actual;
    a.sort();
    let mut b: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    b.sort();
    assert_eq!(a, b);
}

fn pos_of(ordered: &Vec<String>, name: &str) -> usize {
    ordered
        .iter()
        .position(|s| s == name)
        .unwrap_or_else(|| panic!("expected '{}' to be present", name))
}

#[test]
fn test_empty_input() {
    let ordered = get_ordered_files(&[]).unwrap();
    assert_eq!(ordered.len(), 0);
}

#[test]
fn test_independent_files() {
    let files = vec![file("a", r#""#), file("b", r#""#), file("c", r#""#)];
    let ordered = get_ordered_files(&files).unwrap();
    // any order is fine as long as all entries are present
    assert_unordered_eq(ordered, &vec!["a", "b", "c"]);
}

#[test]
fn test_shared_dependency() {
    let files = vec![
        file("a", r#"import common"#),
        file("b", r#"import common"#),
        file("common", r#""#),
    ];
    let ordered = get_ordered_files(&files).unwrap();
    // common must come before both a and b
    let p_common = pos_of(&ordered, "common");
    let p_a = pos_of(&ordered, "a");
    let p_b = pos_of(&ordered, "b");
    assert!(p_common < p_a);
    assert!(p_common < p_b);
    // a and b relative order not important; ensure all present
    assert_unordered_eq(ordered, &vec!["a", "b", "common"]);
}

#[test]
fn test_multiple_imports_and_branching() {
    let files = vec![
        file(
            "main",
            r#"
            import a
            import b
        "#,
        ),
        file("a", r#"import common"#),
        file("b", r#"import common"#),
        file("common", r#""#),
    ];
    let ordered = get_ordered_files(&files).unwrap();
    let p_common = pos_of(&ordered, "common");
    let p_a = pos_of(&ordered, "a");
    let p_b = pos_of(&ordered, "b");
    let p_main = pos_of(&ordered, "main");
    assert!(p_common < p_a);
    assert!(p_common < p_b);
    assert!(p_a < p_main);
    assert!(p_b < p_main);
    assert_unordered_eq(ordered, &vec!["main", "a", "b", "common"]);
}

#[test]
fn test_duplicate_imports_no_error() {
    let files = vec![
        file(
            "dup",
            r#"
            import dep
            import dep
        "#,
        ),
        file("dep", r#""#),
    ];
    let ordered = get_ordered_files(&files).unwrap();
    // dep must come before dup
    assert!(pos_of(&ordered, "dep") < pos_of(&ordered, "dup"));
    assert_unordered_eq(ordered, &vec!["dup", "dep"]);
}

#[test]
fn test_self_import_is_cycle() {
    assert_error(
        &vec![file("selfy", r#"import selfy"#)],
        "import cycle detected",
    );
}
