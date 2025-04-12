package scanner

import (
	"fmt"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/token"
)

type Scanner struct {
	handler *koi.ErrorHandler
	file    *token.File

	src      []byte // Source text to scan
	row      int    // Current line
	col      int    // Current column, resets for each newline
	startCol int    // Column of first character in token
	pos      int    // Pointer to currently scanned character
	base     int    // Pointer to first character of currently scanned token
}

// New makes a new Scanner object for the given file. The text is the raw text
// input to scan. Scanner only accepts ascii text.
func New(file *token.File, text []byte, errHandler *koi.ErrorHandler) *Scanner {
	return &Scanner{
		file:    file,
		src:     text,
		handler: errHandler,
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
	s.startCol = s.col
	return tok
}

func (s *Scanner) err(f string, args ...any) {
	s.handler.Add(fmt.Errorf(f, args...))
}

// peek next byte in input. Return 0 on EOF.
func (s *Scanner) peek() byte {
	if s.pos+1 >= len(s.src) {
		return 0
	}
	return s.src[s.pos+1]
}

func (s *Scanner) next() {
	if s.eof() {
		return
	}

	s.col++
	if s.cur() == '\n' {
		s.row++
		s.col = 0
		s.startCol = 0
	}

	s.pos++
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
		Col:    s.startCol,
		Row:    s.row,
		Offset: s.base,
		File:   s.file,
	}
}

// Get the current position of the Scanner as a token position.
func (s *Scanner) tokenEndPos() token.Pos {
	return token.Pos{
		Col:    s.col,
		Row:    s.row,
		Offset: s.pos,
		File:   s.file,
	}
}

func (s *Scanner) scanWhitespace() token.Token {
	for !s.eof() && isWhitespace(s.cur()) {
		s.next()
		s.base = s.pos
		s.startCol = s.col
	}

	if s.eof() {
		return token.Token{
			Type: token.EOF,
			Eof:  true,
		}
	}

	return s.scanComment()
}

func (s *Scanner) scanComment() token.Token {
	if s.cur() != '/' || s.peek() != '/' {
		return s.scanIdentifier()
	}

	for !s.eof() && s.cur() != '\n' {
		s.next()
	}

	return s.scanWhitespace()
}

func (s *Scanner) scanIdentifier() token.Token {
	if !isAlpha(s.cur()) {
		return s.scanNumber()
	}

	for !s.eof() && isAlpha(s.cur()) {
		s.next()
	}

	str := s.interval()
	typ := token.IDENT

	if t, ok := token.Keywords[str]; ok {
		typ = t
	}

	return token.Token{
		Type:   typ,
		Lexeme: str,
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

	if dots > 1 {
		s.err("number literal can have at most one decimal point")
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

	if !closed {
		s.err("string literals must have a terminating quote on the same line")
	}

	return token.Token{
		Type:    token.STRING,
		Lexeme:  s.interval(),
		Invalid: !closed,
	}
}

func (s *Scanner) scanSymbol() token.Token {
	for sym, typ := range token.DoubleSymbols {
		if s.cur() != sym[0] {
			continue
		}

		if s.peek() == sym[1] {
			s.next()
			s.next()

			return token.Token{
				Type:   typ,
				Lexeme: s.interval(),
			}
		}
	}

	if typ, ok := token.SingleSymbols[string(s.cur())]; ok {
		s.next()
		return token.Token{
			Type:   typ,
			Lexeme: s.interval(),
		}
	}

	return s.scanIllegal()
}

func (s *Scanner) scanIllegal() token.Token {
	s.next()
	s.err("illegal token")

	return token.Token{
		Type:    token.ILLEGAL,
		Invalid: true,
		Lexeme:  s.interval(),
	}
}
