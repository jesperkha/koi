package scanner_test

import (
	"testing"

	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func assertEq(t *testing.T, s *scanner.Scanner, token token.Token) {
	tok := s.Scan()

	if tok.Pos.Col != token.Pos.Col {
		t.Errorf("expected Col=%d, got %d", token.Pos.Col, tok.Pos.Col)
	}
	if tok.Lexeme != token.Lexeme {
		t.Errorf("expected Lexeme=%s, got %s", token.Lexeme, tok.Lexeme)
	}
	if tok.Invalid != token.Invalid {
		t.Errorf("expected Invalid=%v, got %v", token.Invalid, tok.Invalid)
	}
	if tok.Length != token.Length {
		t.Errorf("expected Length=%v, got %v", token.Length, tok.Length)
	}
}

func tok(lexeme string, col int, row int, invalid bool) token.Token {
	return token.Token{
		Lexeme: lexeme,
		Pos: token.Pos{
			Row: row,
			Col: col,
		},
		Length:  len(lexeme),
		Invalid: invalid,
	}
}

func TestScanner(t *testing.T) {
	src := []byte("")
	s := scanner.New(nil, src)

	assertEq(t, s, tok("", 0, 0, false))
}
