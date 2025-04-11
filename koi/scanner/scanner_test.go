package scanner

import (
	"testing"

	"github.com/jesperkha/koi/koi/token"
)

func assertEq(t *testing.T, s *Scanner, token token.Token) {
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

func TestScannerIter(t *testing.T) {
	src := []byte("hello world")
	s := New(token.File{}, src)

	for i, ch := range src {
		if s.eof() {
			t.Error("unexpected eof")
		}

		if ch != s.cur() {
			t.Errorf("expected cur=%c, got %c", ch, s.cur())
		}

		var peek byte
		if i+1 < len(src) {
			peek = src[i+1]
		} else {
			peek = 0
		}

		if peek != s.peek() {
			t.Errorf("expected peek=%c, got %c", peek, s.peek())
		}

		s.consume()
	}

	if !s.eof() {
		t.Error("expected eof")
	}
}
