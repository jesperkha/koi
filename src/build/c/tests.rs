use crate::{
    build::c::emit::emit,
    config::{Config, PathManager},
    util::{FilePath, compare_string_lines_or_panic, emit_string, must},
};

fn emit_src(src: &str) -> String {
    let unit = must(emit_string(src));
    let pm = PathManager::new(FilePath::from(""));
    emit(unit, &Config::test(), &pm).to_string()
}

fn compare(src: &str, expect: &str) {
    compare_string_lines_or_panic(emit_src(src), expect.into());
}

#[test]
fn test_return_0() {
    compare(
        r#"
func main() int {
    return 0
}
        "#,
        r#"
#include "include/koi.h"

int32_t main() {
    return 0;
}
        "#,
    );
}

#[test]
fn test_void_return() {
    compare(
        r#"
func f() {
    return
}
        "#,
        r#"
#include "include/koi.h"

void f() {
    return ;
}
        "#,
    );
}

#[test]
fn test_string_return() {
    compare(
        r#"
func f() string {
    return "Hello"
}
        "#,
        r#"
#include "include/koi.h"

int8_t* f() {
    return "Hello";
}
        "#,
    );
}

#[test]
fn test_function_call() {
    compare(
        r#"
func f(a int, b string) int {
    n := f(1, "Hello")
    return n
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int8_t* t1) {
    int32_t t2 = f(1, "Hello");
    int32_t t3 = t2;
    return t3;
}
        "#,
    );
}

#[test]
fn test_assignment() {
    compare(
        r#"
func f() {
    a := true
    a = false
    b := 1
    b = 0
}
        "#,
        r#"
#include "include/koi.h"

void f() {
    uint8_t t0 = 1;
    t0 = 0;
    int32_t t1 = 1;
    t1 = 0;
    return ;
}
        "#,
    );
}

#[test]
fn test_string_assignment() {
    compare(
        r#"
func f() {
    s := "Hello"
    s = "World"
    x := s
    s = x
}
        "#,
        r#"
#include "include/koi.h"

void f() {
    int8_t* t0 = "Hello";
    t0 = "World";
    int8_t* t1 = t0;
    t0 = t1;
    return ;
}
        "#,
    );
}

#[test]
fn test_binary_add() {
    compare(
        r#"
func f(a int, b int) int {
    return a + b
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int32_t t1) {
    int32_t t2 = t0 + t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_sub() {
    compare(
        r#"
func f(a int, b int) int {
    return a - b
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int32_t t1) {
    int32_t t2 = t0 - t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_mul() {
    compare(
        r#"
func f(a int, b int) int {
    return a * b
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int32_t t1) {
    int32_t t2 = t0 * t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_div() {
    compare(
        r#"
func f(a int, b int) int {
    return a / b
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int32_t t1) {
    int32_t t2 = t0 / t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_eq() {
    compare(
        r#"
func f(a int, b int) bool {
    return a == b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(int32_t t0, int32_t t1) {
    uint8_t t2 = t0 == t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_ne() {
    compare(
        r#"
func f(a int, b int) bool {
    return a != b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(int32_t t0, int32_t t1) {
    uint8_t t2 = t0 != t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_lt() {
    compare(
        r#"
func f(a int, b int) bool {
    return a < b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(int32_t t0, int32_t t1) {
    uint8_t t2 = t0 < t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_gt() {
    compare(
        r#"
func f(a int, b int) bool {
    return a > b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(int32_t t0, int32_t t1) {
    uint8_t t2 = t0 > t1;
    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_and() {
    compare(
        r#"
func f(a bool, b bool) bool {
    return a && b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(uint8_t t0, uint8_t t1) {
    int8_t t2 = 0;
    if (t0) {
        t2 = t1;
    }

    return t2;
}
        "#,
    );
}

#[test]
fn test_binary_or() {
    compare(
        r#"
func f(a bool, b bool) bool {
    return a || b
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(uint8_t t0, uint8_t t1) {
    int8_t t2 = 1;
    if (!t0) {
        t2 = t1;
    }

    return t2;
}
        "#,
    );
}

#[test]
fn test_unary_neg() {
    compare(
        r#"
func f(a int) int {
    return -a
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0) {
    int32_t t1 = -t0;
    return t1;
}
        "#,
    );
}

#[test]
fn test_unary_not() {
    compare(
        r#"
func f(a bool) bool {
    return !a
}
        "#,
        r#"
#include "include/koi.h"

uint8_t f(uint8_t t0) {
    uint8_t t1 = !t0;
    return t1;
}
        "#,
    );
}

#[test]
fn test_param_alloc() {
    compare(
        r#"
func f(a int, b bool, c string) int {
    a = 0
    b = false
    d := 123
    return 0
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, uint8_t t1, int8_t* t2) {
    t0 = 0;
    t1 = 0;
    int32_t t3 = 123;
    return 0;
}
        "#,
    );
}

#[test]
fn test_if_else() {
    compare(
        r#"
func f(a bool) int {
    if a {
        return 0
    } else {
        return 1
    }
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(uint8_t t0) {
    if (t0) {
        return 0;
    } else {
        return 1;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_if_elseif_else() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        return 0
    } else if b {
        return 1
    } else {
        return 2
    }
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(uint8_t t0, uint8_t t1) {
    if (t0) {
        return 0;
    } else {
        if (t1) {
            return 1;
        } else {
            return 2;
        }
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_if_elseif_else_computed() {
    compare(
        r#"
func f(a int, b int) int {
    if a > 0 {
        return 1
    } else if a < b {
        return 2
    } else {
        return 3
    }
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(int32_t t0, int32_t t1) {
    uint8_t t2 = t0 > 0;
    if (t2) {
        return 1;
    } else {
        uint8_t t3 = t0 < t1;
        if (t3) {
            return 2;
        } else {
            return 3;
        }
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_if_elseif_no_else() {
    compare(
        r#"
func f(a bool, b bool) {
    if a {
        return
    } else if b {
        return
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0, uint8_t t1) {
    if (t0) {
        return ;
    } else {
        if (t1) {
            return ;
        }
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_if_else_nested_in_then() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        if b {
            return 0
        } else {
            return 1
        }
    } else {
        return 2
    }
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(uint8_t t0, uint8_t t1) {
    if (t0) {
        if (t1) {
            return 0;
        } else {
            return 1;
        }
    } else {
        return 2;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_if_else_nested_in_else() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        return 0
    } else {
        if b {
            return 1
        } else {
            return 2
        }
    }
}
        "#,
        r#"
#include "include/koi.h"

int32_t f(uint8_t t0, uint8_t t1) {
    if (t0) {
        return 0;
    } else {
        if (t1) {
            return 1;
        } else {
            return 2;
        }
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_simple() {
    compare(
        r#"
func f(a bool) {
    while a {
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0) {
    while (1) {
        if (!t0) {
            break;
        }
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_with_body() {
    compare(
        r#"
func f(a bool) {
    while a {
        a = false
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0) {
    while (1) {
        if (!t0) {
            break;
        }

        t0 = 0;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_computed_condition() {
    compare(
        r#"
func f(a int, b int) {
    while a < b {
        a = a + 1
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(int32_t t0, int32_t t1) {
    while (1) {
        uint8_t t2 = t0 < t1;
        if (!t2) {
            break;
        }

        int32_t t3 = t0 + 1;
        t0 = t3;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_break() {
    compare(
        r#"
func f(a bool) {
    while a {
        break
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0) {
    while (1) {
        if (!t0) {
            break;
        }

        break;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_continue() {
    compare(
        r#"
func f(a bool) {
    while a {
        continue
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0) {
    while (1) {
        if (!t0) {
            break;
        }

        continue;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_break_continue_nested() {
    compare(
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
        r#"
#include "include/koi.h"

void f(uint8_t t0, uint8_t t1) {
    while (1) {
        if (!t0) {
            break;
        }

        while (1) {
            if (!t1) {
                break;
            }

            break;
        }

        continue;
    }

    return ;
}
        "#,
    );
}

#[test]
fn test_while_nested() {
    compare(
        r#"
func f(a bool, b bool) {
    while a {
        while b {
        }
    }
}
        "#,
        r#"
#include "include/koi.h"

void f(uint8_t t0, uint8_t t1) {
    while (1) {
        if (!t0) {
            break;
        }

        while (1) {
            if (!t1) {
                break;
            }
        }
    }

    return ;
}
        "#,
    );
}
