package scanner

import (
	"github.com/jesperkha/koi/koi/token"
)

type Scanner struct {
	file *token.File
	src  []byte // Source text to scan
	row  int    // Current line
	col  int    // Current column, resets for each newline
	pos  int    // Pointer to currently scanned character
	base int    // Pointer to first character of currently scanned token
}

// New makes a new Scanner object for the given file. The text is the raw text
// input to scan. Scanner only accepts ascii text.
func New(file *token.File, text []byte) *Scanner {
	return &Scanner{
		file: file,
		src:  text,
	}
}

// Scan consumes the next token and returns it, advancing the Scanner.
func (s *Scanner) Scan() token.Token {
	if s.eof() {
		return token.Token{
			Eof:  true,
			Type: token.EOF,
		}
	}

	tok := s.scanWhitespace()
	tok.Length = len(tok.Lexeme)
	tok.Pos = s.tokenPos()
	tok.EndPos = s.tokenEndPos()

	s.base = s.pos
	return tok
}

// peek next byte in input. Return 0 on EOF.
func (s *Scanner) peek() byte {
	if s.pos+1 >= len(s.src) {
		return 0
	}
	return s.src[s.pos+1]
}

func (s *Scanner) next() {
	if s.cur() == '\n' {
		s.col = 0
		s.row++
	}

	s.pos++
	s.col++
}

func (s *Scanner) cur() byte {
	if s.eof() {
		return 0
	}
	return s.src[s.pos]
}

func (s *Scanner) eof() bool {
	return s.pos >= len(s.src)
}

func (s *Scanner) interval() string {
	return string(s.src[s.base:s.pos])
}

// Get the current base position of the Scanner as a token position.
func (s *Scanner) tokenPos() token.Pos {
	return token.Pos{
		Col:    s.base,
		Row:    s.row,
		Offset: s.base,
		File:   s.file,
	}
}

// Get the current position of the Scanner as a token position.
func (s *Scanner) tokenEndPos() token.Pos {
	return token.Pos{
		Col:    s.pos,
		Row:    s.row,
		Offset: s.pos,
		File:   s.file,
	}
}

func (s *Scanner) scanWhitespace() token.Token {
	for !s.eof() && isWhitespace(s.cur()) {
		s.next()
		s.base = s.pos
	}

	return s.scanIdentifier()
}

func (s *Scanner) scanIdentifier() token.Token {
	if !isAlpha(s.cur()) {
		return s.scanNumber()
	}

	for !s.eof() && isAlpha(s.cur()) {
		s.next()
	}

	return token.Token{
		Type:   token.IDENT,
		Lexeme: s.interval(),
	}
}

func (s *Scanner) scanNumber() token.Token {
	if !isNum(s.cur()) {
		return s.scanString()
	}

	dots := 0
	typ := token.INTEGER

	for !s.eof() {
		if s.cur() == '.' {
			dots++
			typ = token.FLOAT
		} else if !isNum(s.cur()) {
			break
		}

		s.next()
	}

	return token.Token{
		Type:    typ,
		Lexeme:  s.interval(),
		Invalid: dots > 1,
	}
}

func (s *Scanner) scanString() token.Token {
	if s.cur() != '"' {
		return s.scanSymbol()
	}

	s.next()        // Consume start quote
	closed := false // True if found end quote on current line

	for !s.eof() {
		if s.cur() == '"' {
			s.next() // Consume end quote
			closed = true
			break
		}

		if s.cur() == '\n' {
			break
		}

		s.next()
	}

	return token.Token{
		Type:    token.STRING,
		Lexeme:  s.interval(),
		Invalid: !closed,
	}
}

func (s *Scanner) scanSymbol() token.Token {
	return s.scanIllegal()
}

func (s *Scanner) scanIllegal() token.Token {
	return token.Token{
		Type:    token.ILLEGAL,
		Invalid: true,
		Lexeme:  s.interval(),
	}
}
