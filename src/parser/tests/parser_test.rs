use crate::ast::Printer;
use crate::common::{compare_string_lines_or_panic, must, parse_string};

fn compare_string(src: &str) {
    let ast = must(parse_string(src));
    let pstr = Printer::to_string(&ast);
    compare_string_lines_or_panic(pstr, src.to_string());
}

fn assert_pass(src: &str) {
    let _ = must(parse_string(src));
}

fn expect_error(src: &str, error: &str) {
    if let Err(e) = parse_string(src) {
        assert_eq!(e.len(), 1);
        assert_eq!(e.get(0).message, error);
    } else {
        panic!("expected error");
    }
}

#[test]
fn test_literal_identifiers() {
    compare_string(
        r#"
        func f() {
            foo
            bar
            faz
        }
    "#,
    );
}

#[test]
fn test_literal_grouped() {
    compare_string(
        r#"
        func f() {
            (123)
            ((abc))
        }
    "#,
    );
}

#[test]
fn test_function_with_empty_return() {
    compare_string(
        r#"
        func f() {
            return
        }
    "#,
    );
}

#[test]
fn test_function_with_int_return() {
    compare_string(
        r#"
        func f() int {
            return 0
        }
    "#,
    );
}

#[test]
fn test_function_with_param_and_return() {
    compare_string(
        r#"
        func f(a int) int {
            return 0
        }
    "#,
    );
}

#[test]
fn test_function_with_multiple_params() {
    compare_string(
        r#"
        func f(a int, b bool, c float) int {
            return 0
        }
    "#,
    );
}

#[test]
fn test_function_with_no_return() {
    compare_string(
        r#"
        func f() {
        }
    "#,
    );
}

#[test]
fn test_function_with_bool_return() {
    compare_string(
        r#"
        func f() bool {
            return false
        }
    "#,
    );
}

#[test]
fn test_function_error_missing_body() {
    expect_error(
        r#"
        func f()
    "#,
        "expected {",
    );
}

#[test]
fn test_function_error_unclosed_body() {
    expect_error(
        r#"
        func f() {
    "#,
        "unexpected end of file while parsing block",
    );
}

#[test]
fn test_function_error_missing_close_before_new_func() {
    expect_error(
        r#"
        func f() {

        func g() {}
    "#,
        "expected expression",
    );
}

#[test]
fn test_function_error_missing_param_name() {
    expect_error(
        r#"
        func f( {}
    "#,
        "expected parameter name",
    );
}

#[test]
fn test_function_error_missing_open_paren() {
    expect_error(
        r#"
        func f) {}
    "#,
        "expected (",
    );
}

#[test]
fn test_function_error_param_missing_type() {
    expect_error(
        r#"
        func f(foo) {}
    "#,
        "expected type",
    );
}

#[test]
fn test_function_error_duplicate_param_name() {
    expect_error(
        r#"
        func f(n int, n int) {}
    "#,
        "duplicate parameter name",
    );
}

#[test]
fn test_function_call_no_args() {
    compare_string(
        r#"
        func f() {
            f()
        }
    "#,
    );
}

#[test]
fn test_function_call_one_arg() {
    compare_string(
        r#"
        func f() {
            f(1)
        }
    "#,
    );
}

#[test]
fn test_function_call_multiple_args() {
    compare_string(
        r#"
        func f() {
            f(1, 2, true, abc)
        }
    "#,
    );
}

#[test]
fn test_function_call_nested() {
    compare_string(
        r#"
        func f() {
            a(b(d), b(c(d)))
        }
    "#,
    );
}

#[test]
fn test_complex_function_call_chained() {
    compare_string(
        r#"
        func f() {
            f()()()
        }
    "#,
    );
}

#[test]
fn test_complex_function_call_mixed() {
    compare_string(
        r#"
        func f() {
            a(b()(c))(c, d())
        }
    "#,
    );
}

#[test]
fn test_complex_function_call_deeply_nested() {
    compare_string(
        r#"
        func f() {
            ((a()(b()))()(a()))(a)
        }
    "#,
    );
}

#[test]
fn test_extern_with_return() {
    compare_string(
        r#"
        extern func write(fd int, s string, len int) int
    "#,
    );
}

#[test]
fn test_extern_without_return() {
    compare_string(r#"extern func puts(s string)"#);
}

#[test]
fn test_variable_decl_multiple_types() {
    compare_string(
        r#"
        func f() {
            a := 0
            b := true
            c :: 1.23
        }
    "#,
    );
}

#[test]
fn test_variable_decl_from_variable() {
    compare_string(
        r#"
        func f() {
            a := 0
            b := a
        }
    "#,
    );
}

#[test]
fn test_variable_decl_with_return() {
    compare_string(
        r#"
        func f() int {
            a := 0
            return a
        }
    "#,
    );
}

#[test]
fn test_variable_decl_error_missing_value_assign() {
    expect_error(
        r#"
        func f() {
            a :=
        }
    "#,
        "expected expression",
    );
}

#[test]
fn test_variable_decl_error_missing_value_const() {
    expect_error(
        r#"
        func f() {
            a ::
        }
    "#,
        "expected expression",
    );
}

#[test]
fn test_variable_decl_error_literal_lhs() {
    expect_error(
        r#"
        func f() {
            1 := 1
        }
    "#,
        "invalid left hand value in declaration",
    );
}

#[test]
fn test_variable_decl_error_call_lhs() {
    expect_error(
        r#"
        func f() {
            f() := 1
        }
    "#,
        "invalid left hand value in declaration",
    );
}

#[test]
fn test_variable_assign() {
    compare_string(
        r#"
        func f() {
            a = 0
            b = true
            c = b
        }
    "#,
    );
}

#[test]
fn test_import_module_path() {
    compare_string(
        r#"
        import foo
    "#,
    );
}

#[test]
fn test_import_dotted_path() {
    compare_string(
        r#"
        import foo.bar.faz
    "#,
    );
}

#[test]
fn test_import_alias() {
    compare_string(
        r#"
        import foo as bar
    "#,
    );
}

#[test]
fn test_import_dotted_path_alias() {
    compare_string(
        r#"
        import foo.bar as bar
    "#,
    );
}

#[test]
fn test_import_named_multiline() {
    compare_string(
        r#"
        import foo {
            Foo,
            Bar
        }
    "#,
    );
}

#[test]
fn test_import_named_dotted_multiline() {
    compare_string(
        r#"
        import foo.bar {
            Foo,
            Bar
        }
    "#,
    );
}

#[test]
fn test_import_named_trailing_comma() {
    assert_pass(
        r#"
        import foo.bar{
            Foo,
            Bar, }
    "#,
    );
}

#[test]
fn test_import_named_inline() {
    assert_pass(
        r#"
        import foo { Foo, Bar }
    "#,
    );
}

#[test]
fn test_import_error_alias_after_named() {
    expect_error(
        r#"
        import foo { bar } as faz
    "#,
        "alias is not allowed after named imports",
    );
}

#[test]
fn test_member_simple() {
    assert_pass(
        r#"
        func f() {
            obj.field
        }
    "#,
    );
}

#[test]
fn test_member_chained() {
    assert_pass(
        r#"
        func f() {
            one.two.three.four
        }
    "#,
    );
}

#[test]
fn test_member_with_calls() {
    assert_pass(
        r#"
        func f() {
            one().two.three()
        }
    "#,
    );
}

#[test]
fn test_member_as_argument() {
    assert_pass(
        r#"
        func f() {
            one(two.three, four.five())
        }
    "#,
    );
}

#[test]
fn test_member_error_trailing_dot() {
    expect_error(
        r#"
        func f() {
            one.
        }
    "#,
        "expected field name",
    );
}

#[test]
fn test_member_error_number_after_dot() {
    expect_error(
        r#"
        func f() {
            one.1
        }
    "#,
        "expected field name",
    );
}

#[test]
fn test_member_error_paren_after_dot() {
    expect_error(
        r#"
        func f() {
            one.()
        }
    "#,
        "expected field name",
    );
}

#[test]
fn test_member_error_leading_dot() {
    expect_error(
        r#"
        func f() {
            .one
        }
    "#,
        "expected expression",
    );
}

// Additional edge-case tests

#[test]
fn test_pub_extern_parsed() {
    // ensure public extern declarations are accepted
    assert_pass(r#"pub extern func write(fd int, s string, len int) int"#);
}

#[test]
fn test_unclosed_call_paren_error() {
    expect_error(
        r#"
        func f() {
            a(b
        }
    "#,
        "expected ,",
    );
}

#[test]
fn test_only_newlines_is_ok() {
    // file with only newlines should parse as empty file
    assert_pass("\n\n");
}

#[test]
fn test_recovery_reports_errors_but_continues() {
    let src = r#"
        func bad() {
            .
        }
        func good() {
        }
    "#;
    match parse_string(src) {
        Ok(_) => panic!("expected parse errors"),
        Err(errs) => assert!(errs.len() >= 1),
    }
}

#[test]
fn test_param_list_missing_comma_reports_error() {
    let src = r#"
        func bad(a int b int) {}
    "#;
    match parse_string(src) {
        Ok(_) => panic!("expected parse errors"),
        Err(errs) => assert!(errs.len() >= 1),
    }
}

#[test]
fn test_unclosed_group_reports_error() {
    let src = r#"
        func f() {
            (1
        }
    "#;
    match parse_string(src) {
        Ok(_) => panic!("expected parse errors"),
        Err(errs) => assert!(errs.len() >= 1),
    }
}

#[test]
fn test_pub_alone_reports_error_and_recovers() {
    let src = r#"
        pub

        func good() {}
    "#;
    match parse_string(src) {
        Ok(_) => panic!("expected parse errors"),
        Err(errs) => {
            // Should report at least one error but still attempt recovery
            assert!(errs.len() >= 1);
        }
    }
}

#[test]
fn test_malformed_import_reports_error() {
    let src = r#"
        import foo {
            , bar
        }
        func good() {}
    "#;
    match parse_string(src) {
        Ok(_) => panic!("expected parse errors"),
        Err(errs) => assert!(errs.len() >= 1),
    }
}

#[test]
fn test_binary_addition() {
    assert_pass(
        r#"
        func sum(a int, b int) int {
            return a + b
        }
    "#,
    );
}

#[test]
fn test_binary_subtraction_chained() {
    assert_pass(
        r#"
        func op(a int, b int, c int) int {
            return a + b - c
        }
    "#,
    );
}

#[test]
fn test_binary_grouped() {
    assert_pass(
        r#"
        func f() {
            n + ((a + b) - c)
        }
    "#,
    );
}

#[test]
fn test_binary_mixed_precedence() {
    assert_pass(
        r#"
        func f() {
            1 + 2 * (3 - 4) == 1 - 3
        }
    "#,
    );
}

#[test]
fn test_binary_logical_mixed() {
    assert_pass(
        r#"
        func f() {
            g() && true == false || true
        }
    "#,
    );
}

#[test]
fn test_if_stmt_simple() {
    compare_string(
        r#"
        func f() {
            if true {
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_with_condition() {
    compare_string(
        r#"
        func f() {
            if a == b {
                return
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_with_else() {
    compare_string(
        r#"
        func f() {
            if true {
            } else {
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_with_elseif() {
    compare_string(
        r#"
        func f() {
            if true {
            } else if false {
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_with_elseif_else() {
    compare_string(
        r#"
        func f() {
            if true {
            } else if false {
            } else {
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_with_complex_conditions() {
    compare_string(
        r#"
        func f() {
            if a && b {
                f()
            } else if !a {
                g()
            } else {
                h()
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_error_missing_condition() {
    expect_error(
        r#"
        func f() {
            if {
            }
        }
    "#,
        "expected expression",
    );
}

#[test]
fn test_if_stmt_error_missing_body() {
    expect_error(
        r#"
        func f() {
            if true
        }
    "#,
        "expected {",
    );
}

#[test]
fn test_while_stmt_simple() {
    compare_string(
        r#"
        func f() {
            while true {
            }
        }
    "#,
    );
}

#[test]
fn test_while_stmt_with_condition() {
    compare_string(
        r#"
        func f() {
            while a < b {
                return
            }
        }
    "#,
    );
}

#[test]
fn test_while_stmt_with_body() {
    compare_string(
        r#"
        func f(a bool) {
            while a {
                a = false
            }
        }
    "#,
    );
}

#[test]
fn test_while_stmt_nested() {
    compare_string(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                }
            }
        }
    "#,
    );
}

#[test]
fn test_while_stmt_error_missing_condition() {
    expect_error(
        r#"
        func f() {
            while {
            }
        }
    "#,
        "expected expression",
    );
}

#[test]
fn test_while_stmt_error_missing_body() {
    expect_error(
        r#"
        func f() {
            while true
        }
    "#,
        "expected {",
    );
}

#[test]
fn test_break_stmt() {
    compare_string(
        r#"
        func f() {
            while true {
                break
            }
        }
    "#,
    );
}

#[test]
fn test_continue_stmt() {
    compare_string(
        r#"
        func f() {
            while true {
                continue
            }
        }
    "#,
    );
}

#[test]
fn test_break_continue_in_if_inside_loop() {
    compare_string(
        r#"
        func f(a bool) {
            while a {
                if a {
                    break
                } else {
                    continue
                }
            }
        }
    "#,
    );
}

#[test]
fn test_break_continue_in_nested_loops() {
    compare_string(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                    break
                }
                continue
            }
        }
    "#,
    );
}

#[test]
fn test_unary_not_chained() {
    assert_pass(
        r#"
        func f() {
            !a && !!a && !!!a
        }
    "#,
    );
}

#[test]
fn test_unary_mixed_with_binary() {
    assert_pass(
        r#"
        func f() {
            !a && (-b + n) == !!!!c
        }
    "#,
    );
}

#[test]
fn test_unary_neg_grouped() {
    assert_pass(
        r#"
        func f() {
            -(-a - -b - c)
        }
    "#,
    );
}

#[test]
fn test_nomangle_modifier_on_func() {
    compare_string(
        r#"
        @nomangle
        func f() {
        }
    "#,
    );
}

#[test]
fn test_inline_modifier_on_func() {
    compare_string(
        r#"
        @inline
        func f() {
        }
    "#,
    );
}

#[test]
fn test_naked_modifier_on_func() {
    compare_string(
        r#"
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_two_modifiers_on_func() {
    compare_string(
        r#"
        @inline
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_three_modifiers_on_func() {
    compare_string(
        r#"
        @nomangle
        @inline
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_modifier_on_extern_single() {
    compare_string(
        r#"
        @nomangle
        extern func write(fd int, s string, len int) int
    "#,
    );
}

#[test]
fn test_modifier_on_extern_multiple() {
    compare_string(
        r#"
        @inline
        @naked
        extern func puts(s string)
    "#,
    );
}

#[test]
fn test_modifier_on_pub_func_single() {
    assert_pass(
        r#"
        @nomangle
        pub func f() {
        }
    "#,
    );
}

#[test]
fn test_modifier_on_pub_func_multiple() {
    assert_pass(
        r#"
        @inline
        @naked
        pub func f() {
        }
    "#,
    );
}

#[test]
fn test_modifier_on_pub_extern() {
    assert_pass(
        r#"
        @nomangle
        pub extern func write(fd int, s string, len int) int
    "#,
    );
}

#[test]
fn test_modifier_missing_decl_error() {
    expect_error(
        r#"
        @nomangle

        func f() {
        }
    "#,
        "expected declaration after modifier",
    );
}

#[test]
fn test_modifier_missing_name_error() {
    expect_error(
        r#"
        @
        func f() {
        }
    "#,
        "expected modifier name",
    );
}

#[test]
fn test_for_loop_basic() {
    assert_pass(
        r#"
        func f() {
            for i := 0; i < 10; i = i + 1 {
                g()
            }
        }
    "#,
    );
}

#[test]
fn test_for_loop_with_call_in_init_and_post() {
    assert_pass(
        r#"
        func f() {
            for i := foo(0); true; i = foo(i) {

            }
        }
    "#,
    );
}

#[test]
fn test_complex_for_loop_expressions_with_calls() {
    assert_pass(
        r#"
        func f() {
            for println("Hello"); true; return 0 {
                println("World")
            }
        }
    "#,
    );
}

#[test]
fn test_complex_for_loop_expressions_nested() {
    assert_pass(
        r#"
        func f() {
            for if true {
                return
            }; false; for i := 0; i < 10; i = i + 1 {
                break
            } {
                g()
            }
        }
    "#,
    );
}

// --- Type declarations ---

#[test]
fn test_type_decl_basic() {
    compare_string(r#"type Number int"#);
}

#[test]
fn test_type_decl_unique() {
    compare_string(r#"unique type ID u64"#);
}

#[test]
fn test_type_decl_pub() {
    compare_string(r#"pub type Foo int"#);
}

#[test]
fn test_type_decl_pub_unique() {
    compare_string(r#"pub unique type ID u64"#);
}

#[test]
fn test_type_decl_imported_base_type() {
    compare_string(r#"type Foo bar.Baz"#);
}

#[test]
fn test_type_decl_missing_name_error() {
    expect_error(r#"type 123 int"#, "expected type name");
}

#[test]
fn test_type_decl_missing_underlying_type_error() {
    expect_error(
        r#"
        type Foo 123
    "#,
        "invalid type",
    );
}

#[test]
fn test_unique_type_decl_missing_type_keyword_error() {
    expect_error(r#"unique Foo int"#, "expected type");
}

// --- Cast expressions ---

#[test]
fn test_cast_basic() {
    compare_string(
        r#"
        func f() {
            x as i32
        }
    "#,
    );
}

#[test]
fn test_cast_in_return() {
    compare_string(
        r#"
        func f() i32 {
            return x as i32
        }
    "#,
    );
}

#[test]
fn test_cast_of_literal() {
    compare_string(
        r#"
        func f() i64 {
            return 123 as i64
        }
    "#,
    );
}

#[test]
fn test_cast_in_var_decl() {
    compare_string(
        r#"
        func f() {
            x := foo as u8
        }
    "#,
    );
}

#[test]
fn test_cast_imported_type() {
    compare_string(
        r#"
        func f() {
            x as ns.MyType
        }
    "#,
    );
}

#[test]
fn test_cast_precedence_with_add() {
    // `as` binds tighter than `+`: parses and prints as `a + b as i32`
    compare_string(
        r#"
        func f() {
            a + b as i32
        }
    "#,
    );
}

#[test]
fn test_cast_after_unary() {
    // unary is applied before cast: `-x as i32` == `(-x) as i32`
    compare_string(
        r#"
        func f() {
            -x as i32
        }
    "#,
    );
}

#[test]
fn test_cast_missing_type_error() {
    expect_error(
        r#"
        func f() {
            x as
        }
    "#,
        "invalid type",
    );
}
