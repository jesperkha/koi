package scanner

import (
	"fmt"
	"strings"

	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Scanner struct {
	// Number of syntax errors encountered and reported to handler.
	NumErrors int

	errors util.ErrorList
	file   *token.File

	src       []byte // Source text to scan
	row       int    // Current line
	col       int    // Current column, resets for each newline
	startCol  int    // Column of first character in token
	pos       int    // Pointer to currently scanned character
	base      int    // Pointer to first character of currently scanned token
	lineBegin int    // First character on current line
}

// New makes a new Scanner object for the given file. The src is the raw text
// input to scan. Scanner only accepts ascii text.
func New(file *token.File, src []byte) *Scanner {
	return &Scanner{
		file:   file,
		src:    src,
		errors: util.ErrorList{},
	}
}

// Scan consumes the next token and returns it, advancing the Scanner.
func (s *Scanner) Scan() token.Token {
	if s.eof() {
		return s.eofToken()
	}

	tok := s.scanWhitespace()

	s.base = s.pos
	s.startCol = s.col
	return tok
}

// ScanAll scans input source until everything is tokenized.
func (s *Scanner) ScanAll() []token.Token {
	toks := []token.Token{}
	var t token.Token

	for !t.Eof {
		t = s.Scan()
		toks = append(toks, t)
	}

	return toks
}

func (s *Scanner) Error() error {
	return s.errors.Error()
}

func isAlpha(c byte) bool {
	return strings.Contains("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_", string(c))
}

func isNum(c byte) bool {
	return strings.Contains("0123456789", string(c))
}

func isWhitespace(c byte) bool {
	return strings.Contains("\t\r ", string(c))
}

func (s *Scanner) eofToken() token.Token {
	return token.Token{
		Type:   token.EOF,
		Eof:    true,
		Pos:    s.tokenPos(),
		EndPos: s.tokenEndPos(),
	}
}

func (s *Scanner) err(f string, args ...any) {
	lineStr := s.src[s.lineBegin : util.FindEndOfLine(s.src, s.lineBegin)+1]
	length := s.col - s.startCol

	err := ""
	err += fmt.Sprintf("error: %s\n", fmt.Sprintf(f, args...))
	err += fmt.Sprintf("%3d | %s\n", s.row+1, lineStr)
	err += fmt.Sprintf("    | %s%s\n", strings.Repeat(" ", s.startCol), strings.Repeat("^", length))

	s.errors.Add(fmt.Errorf("%s", err))
	s.NumErrors++
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
		s.base = s.pos + 1
		s.lineBegin = s.pos + 1
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
		Col:       s.startCol,
		Row:       s.row,
		Offset:    s.base,
		File:      s.file,
		LineBegin: s.lineBegin,
	}
}

// Get the current position of the Scanner as a token position.
func (s *Scanner) tokenEndPos() token.Pos {
	return token.Pos{
		Col:       s.col,
		Row:       s.row,
		Offset:    s.pos,
		File:      s.file,
		LineBegin: s.lineBegin,
	}
}

func (s *Scanner) scanWhitespace() token.Token {
	for !s.eof() && isWhitespace(s.cur()) {
		s.next()
		s.base = s.pos
		s.startCol = s.col
	}

	if s.eof() {
		return s.eofToken()
	}

	return s.scanNewline()
}

func (s *Scanner) scanNewline() token.Token {
	if s.cur() == '\n' {
		pos := s.tokenPos()
		endPos := s.tokenEndPos()
		s.next()
		return token.Token{
			Type:   token.NEWLINE,
			Lexeme: "NEWLINE",
			Pos:    pos,
			EndPos: endPos,
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

	// Newline tokens after comments have no
	// semantic value so we can just ignore them.
	s.next()
	return s.scanWhitespace()
}

func (s *Scanner) scanIdentifier() token.Token {
	if !isAlpha(s.cur()) {
		return s.scanNumber()
	}

	for !s.eof() && (isAlpha(s.cur()) || isNum(s.cur())) {
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
		Pos:    s.tokenPos(),
		EndPos: s.tokenEndPos(),
	}
}

func (s *Scanner) scanNumber() token.Token {
	if !isNum(s.cur()) {
		return s.scanString()
	}

	dots := 0

	for !s.eof() {
		if s.cur() == '.' {
			dots++
		} else if !isNum(s.cur()) {
			break
		}

		s.next()
	}

	if dots > 1 {
		s.err("number literal can have at most one decimal point")
	}

	return token.Token{
		Type:    token.NUMBER,
		Lexeme:  s.interval(),
		Invalid: dots > 1,
		Float:   dots > 0,
		Pos:     s.tokenPos(),
		EndPos:  s.tokenEndPos(),
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
		Pos:     s.tokenPos(),
		EndPos:  s.tokenEndPos(),
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
				Pos:    s.tokenPos(),
				EndPos: s.tokenEndPos(),
			}
		}
	}

	if typ, ok := token.SingleSymbols[string(s.cur())]; ok {
		s.next()
		return token.Token{
			Type:   typ,
			Lexeme: s.interval(),
			Pos:    s.tokenPos(),
			EndPos: s.tokenEndPos(),
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
		Pos:     s.tokenPos(),
		EndPos:  s.tokenEndPos(),
	}
}
