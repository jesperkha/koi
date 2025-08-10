use std::vec;

use super::*;
use crate::token::{File, Token, TokenKind};

fn scan_source(s: &str) -> ScannerResult {
    let file = File::new_test_file(s);
    Scanner::new(&file).scan()
}

fn scan_and_then<P>(src: &str, pred: P)
where
    P: Fn(Vec<Token>),
{
    let _ = scan_source(src).map_err(|e| panic!("{:?}", e)).map(pred);
}

fn scan_and_error(src: &str) {
    assert!(scan_source(src).is_err());
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
fn test_number() {
    let _ = scan_and_then("123", |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].length, 3);
        assert_eq!(toks[0].kind, TokenKind::IntLit(123));
    });

    let _ = scan_and_then("1.23", |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].length, 4);
        assert_eq!(toks[0].kind, TokenKind::FloatLit(1.23));
    });

    let _ = scan_source("?123?")
        .map_err(|e| panic!("{:?}", e))
        .map(|toks| {
            assert_eq!(toks.len(), 3);
            assert_eq!(toks[1].kind, TokenKind::IntLit(123));
        });

    if scan_source("1.2.3").is_ok() {
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
fn test_string() {
    scan_and_then(r#""Hello world!""#, |toks| {
        assert_eq!(toks.len(), 1);
        assert_eq!(
            toks[0].kind,
            TokenKind::StringLit("Hello world!".to_string())
        );
        assert_eq!(toks[0].length, 14);
    });

    scan_and_then("goodbye\"cruel\"world", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[1].kind, TokenKind::StringLit("cruel".to_string()));
    });

    scan_and_error("\"not terminated, no newline");
    scan_and_error("\"with newline\n123");
}

#[test]
fn test_symbols() {
    scan_and_then("+ - = /", |toks| {
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenKind::Plus);
        assert_eq!(toks[1].kind, TokenKind::Minus);
        assert_eq!(toks[2].kind, TokenKind::Eq);
        assert_eq!(toks[3].kind, TokenKind::Slash);
    });

    scan_and_then("+= /= >= :=", |toks| {
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenKind::PlusEq);
        assert_eq!(toks[1].kind, TokenKind::SlashEq);
        assert_eq!(toks[2].kind, TokenKind::GreaterEq);
        assert_eq!(toks[3].kind, TokenKind::ColonEq);
    });

    scan_and_then("+-=/", |toks| {
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::Plus);
        assert_eq!(toks[1].kind, TokenKind::MinusEq);
        assert_eq!(toks[2].kind, TokenKind::Slash);
    });
}

#[test]
fn test_line_comment() {
    scan_and_then("hello // comment\nworld", |toks| {
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("hello".to_string()));
        assert_eq!(toks[1].kind, TokenKind::IdentLit("world".to_string()));
    });

    scan_and_then("// comment", |toks| assert_eq!(toks.len(), 0));
    scan_and_then("// comment\n", |toks| assert_eq!(toks.len(), 0));
}

#[test]
fn test_block_comment() {
    let expect_foo = |toks: Vec<Token>| {
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::IdentLit("foo".to_string()));
    };

    scan_and_then("/* comment */foo", expect_foo);
    scan_and_then("/* nested \n /* comment */\n */ foo", expect_foo);
    scan_and_then("foo /* nested /* comment\n */ */", expect_foo);

    scan_and_error("/* not closed");
    scan_and_error("/* not closed /* nested */");
}

#[test]
fn test_illegal_token() {
    scan_and_error("@");
}
