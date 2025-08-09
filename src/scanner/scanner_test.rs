use std::vec;

use super::*;
use crate::token::TokenKind;

fn scanner_from(s: &str) -> Scanner {
    Scanner::new(s.to_string().into_bytes())
}

#[test]
fn test_whitespace_only() {
    let mut s = scanner_from("  \t \t   \r  ");
    match s.scan() {
        Ok(toks) => assert_eq!(toks.len(), 0),
        Err(e) => panic!("{:?}", e),
    };
}

#[test]
fn test_identifier() {
    let expect = vec!["foo", "bar", "a", "abc_123"];
    let mut s = scanner_from(expect.join(" ").as_str());

    match s.scan() {
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
fn test_pos() {
    let mut s = scanner_from("abc def\nhello world");
    let expect_pos = vec![(0, 0), (0, 4), (0, 7), (1, 0), (1, 6)]; // (row, col)
    let expect_end = vec![(0, 3), (0, 7), (0, 8), (1, 5), (1, 11)]; // (row, col)
    let expect_offset = vec![0, 4, 7, 8, 14];
    let expect_line = vec![0, 0, 0, 8, 8];

    match s.scan() {
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
