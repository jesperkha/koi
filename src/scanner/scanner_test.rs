use std::vec;

use super::*;
use crate::token::{File, TokenKind};

fn scan_source(s: &str) -> ScannerResult {
    let file = File::new_test_file(s);
    Scanner::new(&file).scan()
}

#[test]
fn test_whitespace_only() {
    match scan_source("  \t \t   \r  ") {
        Ok(toks) => assert_eq!(toks.len(), 0),
        Err(e) => panic!("{:?}", e),
    };
}

#[test]
fn test_identifier() {
    let expect = vec!["foo", "bar", "a", "abc_123"];
    match scan_source(expect.join(" ").as_str()) {
        Ok(toks) => {
            assert_eq!(toks.len(), 4);
            for (i, t) in toks.iter().enumerate() {
                assert!(!t.eof && !t.invalid);
                assert_eq!(t.length, expect[i].len());
                assert_eq!(t.kind, TokenKind::IdentLit(expect[i].to_string()));
            }
        }
        Err(e) => panic!("{:?}", e),
    };
}

#[test]
fn test_number() {
    match scan_source("123") {
        Ok(toks) => {
            assert_eq!(toks.len(), 1);
            assert_eq!(toks[0].length, 3);
            assert_eq!(toks[0].kind, TokenKind::IntLit(123));
        }
        Err(e) => panic!("{:?}", e),
    }
    match scan_source("1.23") {
        Ok(toks) => {
            assert_eq!(toks.len(), 1);
            assert_eq!(toks[0].length, 4);
            assert_eq!(toks[0].kind, TokenKind::FloatLit(1.23));
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[test]
fn test_pos() {
    let expect_pos = vec![(0, 0), (0, 4), (0, 7), (1, 0), (1, 6)]; // (row, col)
    let expect_end = vec![(0, 3), (0, 7), (0, 8), (1, 5), (1, 11)]; // (row, col)
    let expect_offset = vec![0, 4, 7, 8, 14];
    let expect_line = vec![0, 0, 0, 8, 8];

    match scan_source("abc def\nhello world") {
        Ok(toks) => {
            assert_eq!(toks.len(), expect_pos.len());
            for (i, t) in toks.iter().enumerate() {
                assert_eq!(t.pos.row, expect_pos[i].0, "case {}", i + 1);
                assert_eq!(t.pos.col, expect_pos[i].1, "case {}", i + 1);

                assert_eq!(t.end_pos.row, expect_end[i].0, "case {}", i + 1);
                assert_eq!(t.end_pos.col, expect_end[i].1, "case {}", i + 1);

                assert_eq!(t.pos.offset, expect_offset[i], "case {}", i + 1);
                assert_eq!(t.pos.line_begin, expect_line[i], "case {}", i + 1);
            }
        }
        Err(e) => panic!("{:?}", e),
    };
}
