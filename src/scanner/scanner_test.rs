use std::vec;

use crate::{
    ast::{Token, TokenKind},
    util::{must, scan_string},
};

fn scan_and_then<P>(src: &str, pred: P)
where
    P: Fn(Vec<Token>),
{
    pred(must(scan_string(src)));
}

fn scan_and_error(src: &str) {
    assert!(scan_string(src).is_err());
}

#[test]
fn test_whitespace_only() {
    scan_and_then("  \t \t   \r  ", |toks| assert_eq!(toks.len(), 0));
}

#[test]
fn test_identifier() {
    let expect = vec!["foo", "bar", "a", "abc_123"];
    let _ = scan_and_then(expect.join(" ").as_str(), |toks| {
        assert_eq!(toks.len(), 4);
        for (i, t) in toks.iter().enumerate() {
            assert!(!t.eof && !t.invalid);
            assert_eq!(t.length, expect[i].len());
            assert_eq!(t.kind, TokenKind::IdentLit(expect[i].to_string()));
        }
    });
}

#[test]
fn test_keywords() {
    scan_and_then("true false pub import", |toks| {
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenKind::True);
        assert_eq!(toks[1].kind, TokenKind::False);
        assert_eq!(toks[2].kind, TokenKind::Pub);
        assert_eq!(toks[3].kind, TokenKind::Import);
    });
}

#[test]
fn test_number_integer() {
    scan_and_then("123", |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].length, 3);
        assert_eq!(toks[0].kind, TokenKind::IntLit(123));
    });
}

#[test]
fn test_number_float() {
    scan_and_then("1.23", |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].length, 4);
        assert_eq!(toks[0].kind, TokenKind::FloatLit(1.23));
    });
}

#[test]
fn test_number_between_symbols() {
    scan_and_then("?123?", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[1].kind, TokenKind::IntLit(123));
    });
}

#[test]
fn test_number_double_decimal_error() {
    if scan_string("1.2.3").is_ok() {
        panic!("expected scanner error");
    }
}

#[test]
fn test_pos() {
    let expect_pos = vec![(0, 0), (0, 4), (0, 7), (1, 0), (1, 6)]; // (row, col)
    let expect_end = vec![(0, 3), (0, 7), (0, 8), (1, 5), (1, 11)]; // (row, col)
    let expect_offset = vec![0, 4, 7, 8, 14];
    let expect_line = vec![0, 0, 0, 8, 8];

    scan_and_then("abc def\nhello world", |toks| {
        assert_eq!(toks.len(), expect_pos.len());
        for (i, t) in toks.iter().enumerate() {
            assert_eq!(t.pos.row, expect_pos[i].0, "case {}", i + 1);
            assert_eq!(t.pos.col, expect_pos[i].1, "case {}", i + 1);

            assert_eq!(t.end_pos.row, expect_end[i].0, "case {}", i + 1);
            assert_eq!(t.end_pos.col, expect_end[i].1, "case {}", i + 1);

            assert_eq!(t.pos.offset, expect_offset[i], "case {}", i + 1);
            assert_eq!(t.pos.line_begin, expect_line[i], "case {}", i + 1);
        }
    });
}

#[test]
fn test_string_basic() {
    scan_and_then(r#""Hello world!""#, |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(
            toks[0].kind,
            TokenKind::StringLit("Hello world!".to_string())
        );
        assert_eq!(toks[0].length, 14);
    });
}

#[test]
fn test_string_embedded_in_identifiers() {
    scan_and_then("goodbye\"cruel\"world", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[1].kind, TokenKind::StringLit("cruel".to_string()));
    });
}

#[test]
fn test_string_unterminated_no_newline_error() {
    scan_and_error("\"not terminated, no newline");
}

#[test]
fn test_string_unterminated_with_newline_error() {
    scan_and_error("\"with newline\n123");
}

#[test]
fn test_byte_string_valid() {
    scan_and_then(r#"'A'"#, |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::StringLit("A".to_string()));
        assert_eq!(toks[0].length, 3);
    });
}

#[test]
fn test_byte_string_too_long_error() {
    scan_and_error("'too long'");
}

#[test]
fn test_byte_string_empty_error() {
    scan_and_error("''");
}

#[test]
fn test_byte_string_unterminated_error() {
    scan_and_error("'a");
}

#[test]
fn test_symbols_basic() {
    scan_and_then("+ - = /", |toks| {
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenKind::Plus);
        assert_eq!(toks[1].kind, TokenKind::Minus);
        assert_eq!(toks[2].kind, TokenKind::Eq);
        assert_eq!(toks[3].kind, TokenKind::Slash);
    });
}

#[test]
fn test_symbols_compound() {
    scan_and_then("+= /= >= :=", |toks| {
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenKind::PlusEq);
        assert_eq!(toks[1].kind, TokenKind::SlashEq);
        assert_eq!(toks[2].kind, TokenKind::GreaterEq);
        assert_eq!(toks[3].kind, TokenKind::ColonEq);
    });
}

#[test]
fn test_symbols_adjacent_greedy() {
    scan_and_then("+-=/", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::Plus);
        assert_eq!(toks[1].kind, TokenKind::MinusEq);
        assert_eq!(toks[2].kind, TokenKind::Slash);
    });
}

#[test]
fn test_line_comment_inline() {
    scan_and_then("hello // comment\nworld", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("hello".to_string()));
        assert_eq!(toks[2].kind, TokenKind::IdentLit("world".to_string()));
    });
}

#[test]
fn test_line_comment_only() {
    scan_and_then("// comment", |toks| assert_eq!(toks.len(), 0));
}

#[test]
fn test_block_comment_before_token() {
    let expect_foo = |toks: Vec<Token>| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("foo".to_string()));
    };
    scan_and_then("/* comment */foo", expect_foo);
}

#[test]
fn test_block_comment_nested_before_token() {
    let expect_foo = |toks: Vec<Token>| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("foo".to_string()));
    };
    scan_and_then("/* nested \n /* comment */\n */ foo", expect_foo);
}

#[test]
fn test_block_comment_nested_after_token() {
    let expect_foo = |toks: Vec<Token>| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("foo".to_string()));
    };
    scan_and_then("foo /* nested /* comment\n */ */", expect_foo);
}

#[test]
fn test_block_comment_unclosed_error() {
    scan_and_error("/* not closed");
}

#[test]
fn test_block_comment_unclosed_nested_error() {
    scan_and_error("/* not closed /* nested */");
}

#[test]
fn test_hex_literal_lowercase() {
    scan_and_then("0xff", |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::IntLit(255));
        assert_eq!(toks[0].length, 4);
    });
}

#[test]
fn test_hex_literal_uppercase_digits() {
    scan_and_then("0xFF", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(255));
    });
}

#[test]
fn test_hex_literal_uppercase_prefix() {
    scan_and_then("0XFF", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(255));
    });
}

#[test]
fn test_hex_literal_zero() {
    scan_and_then("0x0", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(0));
    });
}

#[test]
fn test_hex_literal_multi_digit() {
    scan_and_then("0x1A2B", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(0x1A2B));
        assert_eq!(toks[0].length, 6);
    });
}

#[test]
fn test_hex_literal_surrounded_by_tokens() {
    scan_and_then("a + 0xDEAD", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[2].kind, TokenKind::IntLit(0xDEAD));
    });
}

#[test]
fn test_hex_literal_no_digits_lowercase_error() {
    scan_and_error("0x");
}

#[test]
fn test_hex_literal_no_digits_uppercase_error() {
    scan_and_error("0X");
}

#[test]
fn test_hex_literal_zero_is_int() {
    scan_and_then("0", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(0));
    });
}

#[test]
fn test_hex_literal_zero_followed_by_digit_is_int() {
    scan_and_then("01", |toks| {
        assert_eq!(toks[0].kind, TokenKind::IntLit(1));
    });
}
