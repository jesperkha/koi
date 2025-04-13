package scanner

import (
	"testing"

	"github.com/jesperkha/koi/koi/token"
)

func TestScannerIter(t *testing.T) {
	src := []byte("hello world")
	s := New(&token.File{}, src)

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

		s.next()
	}

	if !s.eof() {
		t.Error("expected eof")
	}
}

func TestFindEndOfLine(t *testing.T) {
	src := []byte("hello there\nmy name is bob")

	// offset: expect
	cases := map[int]int{
		5:  10,
		14: 25,
	}

	for k, v := range cases {
		if n := findEndOfLine(src, k); n != v {
			t.Errorf("expected end=%d, got %d, for offset=%d", v, n, k)
		}
	}
}

func assertEq(t *testing.T, s *Scanner, token token.Token) {
	tok := s.Scan()

	if tok.Pos.Col != token.Pos.Col {
		t.Errorf("'%s': expected Col=%d, got %d", token.Lexeme, token.Pos.Col, tok.Pos.Col)
	}
	if tok.Lexeme != token.Lexeme {
		t.Errorf("'%s': expected Lexeme=%s, got %s", token.Lexeme, token.Lexeme, tok.Lexeme)
	}
	if tok.Invalid != token.Invalid {
		t.Errorf("'%s': expected Invalid=%v, got %v", token.Lexeme, token.Invalid, tok.Invalid)
	}
	if tok.Length != token.Length {
		t.Errorf("'%s': expected Length=%v, got %v", token.Lexeme, token.Length, tok.Length)
	}
	if tok.Type != token.Type {
		t.Errorf("'%s': expected Type=%d, got %d", token.Lexeme, token.Type, tok.Type)
	}
}

func assertEof(t *testing.T, s *Scanner) {
	if !s.eof() {
		t.Error("expected eof after test")
	}
}

func tok(typ token.TokenType, lexeme string, col int, row int, invalid bool) token.Token {
	return token.Token{
		Lexeme: lexeme,
		Type:   typ,
		Pos: token.Pos{
			Row: row,
			Col: col,
		},
		Length:  len(lexeme),
		Invalid: invalid,
	}
}

func TestScannerIdent(t *testing.T) {
	src := []byte("hello foo_bar john123")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "hello", 0, 0, false))
	assertEq(t, s, tok(token.IDENT, "foo_bar", 6, 0, false))
	assertEq(t, s, tok(token.IDENT, "john123", 14, 0, false))
	assertEof(t, s)
}

func TestScannerKeyword(t *testing.T) {
	src := []byte("none nil preturn elsee")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "none", 0, 0, false))
	assertEq(t, s, tok(token.NIL, "nil", 5, 0, false))
	assertEq(t, s, tok(token.IDENT, "preturn", 9, 0, false))
	assertEq(t, s, tok(token.IDENT, "elsee", 17, 0, false))
	assertEof(t, s)
}

func TestScannerNumber(t *testing.T) {
	src := []byte("123 1.23")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.INTEGER, "123", 0, 0, false))
	assertEq(t, s, tok(token.FLOAT, "1.23", 4, 0, false))
	assertEof(t, s)

	src = []byte("1.1.2 123..4")
	s = New(&token.File{}, src)

	assertEq(t, s, tok(token.FLOAT, "1.1.2", 0, 0, true))
	assertEq(t, s, tok(token.FLOAT, "123..4", 6, 0, true))
	assertEof(t, s)
}

func TestScannerString(t *testing.T) {
	src := []byte("\"hello\" \"there\"")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.STRING, "\"hello\"", 0, 0, false))
	assertEq(t, s, tok(token.STRING, "\"there\"", 8, 0, false))
	assertEof(t, s)

	src = []byte("\"no end quote")
	s = New(&token.File{}, src)

	assertEq(t, s, tok(token.STRING, "\"no end quote", 0, 0, true))
	assertEof(t, s)
}

func TestScannerSymbol(t *testing.T) {
	src := []byte("++= == /= !!=")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.PLUS, "+", 0, 0, false))
	assertEq(t, s, tok(token.PLUS_EQ, "+=", 1, 0, false))
	assertEq(t, s, tok(token.EQ_EQ, "==", 4, 0, false))
	assertEq(t, s, tok(token.DIV_EQ, "/=", 7, 0, false))
	assertEq(t, s, tok(token.NOT, "!", 10, 0, false))
	assertEq(t, s, tok(token.NOT_EQ, "!=", 11, 0, false))
	assertEof(t, s)

	src = []byte("?^$")
	s = New(&token.File{}, src)

	assertEq(t, s, tok(token.ILLEGAL, "?", 0, 0, true))
	assertEq(t, s, tok(token.ILLEGAL, "^", 1, 0, true))
	assertEq(t, s, tok(token.ILLEGAL, "$", 2, 0, true))
	assertEof(t, s)
}

func TestScannerWhitespace(t *testing.T) {
	src := []byte("   \t\n  hello   \n\tworld  ")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "hello", 2, 1, false))
	assertEq(t, s, tok(token.IDENT, "world", 1, 2, false))
	assertEq(t, s, tok(token.EOF, "", 8, 2, false))
	assertEof(t, s)
}

func TestScannerNewlines(t *testing.T) {
	src := []byte("a\nb\n\nc")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "a", 0, 0, false))
	assertEq(t, s, tok(token.IDENT, "b", 0, 1, false))
	assertEq(t, s, tok(token.IDENT, "c", 0, 3, false))
	assertEof(t, s)
}

func TestScannerMixedTokens(t *testing.T) {
	src := []byte("var x = 42 + 3.14;")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "var", 0, 0, false))
	assertEq(t, s, tok(token.IDENT, "x", 4, 0, false))
	assertEq(t, s, tok(token.EQ, "=", 6, 0, false))
	assertEq(t, s, tok(token.INTEGER, "42", 8, 0, false))
	assertEq(t, s, tok(token.PLUS, "+", 11, 0, false))
	assertEq(t, s, tok(token.FLOAT, "3.14", 13, 0, false))
	assertEq(t, s, tok(token.SEMI, ";", 17, 0, false))
	assertEof(t, s)
}

func TestScannerComplexSymbols(t *testing.T) {
	src := []byte("<= >= && || != ==")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.LESS_EQ, "<=", 0, 0, false))
	assertEq(t, s, tok(token.GREATER_EQ, ">=", 3, 0, false))
	assertEq(t, s, tok(token.AND_AND, "&&", 6, 0, false))
	assertEq(t, s, tok(token.OR_OR, "||", 9, 0, false))
	assertEq(t, s, tok(token.NOT_EQ, "!=", 12, 0, false))
	assertEq(t, s, tok(token.EQ_EQ, "==", 15, 0, false))
	assertEof(t, s)
}

func TestScannerInvalidNumber(t *testing.T) {
	src := []byte("123..456")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.FLOAT, "123..456", 0, 0, true))
	assertEof(t, s)
}

func TestScannerEmptySource(t *testing.T) {
	src := []byte("")
	s := New(&token.File{}, src)

	if !s.eof() {
		t.Error("expected eof for empty source")
	}
}

func TestScannerOnlyWhitespace(t *testing.T) {
	src := []byte("  \t \n\t ")
	s := New(&token.File{}, src)

	s.Scan()

	if !s.eof() {
		t.Error("expected eof for whitespace-only source")
	}
}

func TestScannerOnlyComment(t *testing.T) {
	src := []byte("// one comment\n// two comments")
	s := New(&token.File{}, src)

	s.Scan()

	if !s.eof() {
		t.Error("expected eof for comment-only source")
	}
}

func TestScannerPunctuation(t *testing.T) {
	src := []byte(".,:;(){}[]")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.DOT, ".", 0, 0, false))
	assertEq(t, s, tok(token.COMMA, ",", 1, 0, false))
	assertEq(t, s, tok(token.COLON, ":", 2, 0, false))
	assertEq(t, s, tok(token.SEMI, ";", 3, 0, false))
	assertEq(t, s, tok(token.LPAREN, "(", 4, 0, false))
	assertEq(t, s, tok(token.RPAREN, ")", 5, 0, false))
	assertEq(t, s, tok(token.LBRACE, "{", 6, 0, false))
	assertEq(t, s, tok(token.RBRACE, "}", 7, 0, false))
	assertEq(t, s, tok(token.LBRACK, "[", 8, 0, false))
	assertEq(t, s, tok(token.RBRACK, "]", 9, 0, false))
	assertEof(t, s)
}

func TestScannerComment(t *testing.T) {
	src := []byte("// this is a comment\n  // another one\nvar//foo\n123")
	s := New(&token.File{}, src)

	assertEq(t, s, tok(token.IDENT, "var", 0, 2, false))
	assertEq(t, s, tok(token.INTEGER, "123", 0, 3, false))
	assertEof(t, s)
}
