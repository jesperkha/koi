package scanner

import "github.com/jesperkha/koi/koi/token"

type Scanner struct {
	file      token.File
	numErrors int
	text      []byte
	line      int
	offset    int
	ch        byte // Current character
}

// New makes a new Scanner object for the given file. The text is the raw text
// input to scan. Scanner only accepts ascii text.
func New(file token.File, text []byte) *Scanner {
	return &Scanner{
		file:      file,
		text:      text,
		numErrors: 0,
		line:      0,
		offset:    0,
		ch:        0,
	}
}

// Scan consumes the next token and returns it, advancing the Scanner.
func (s *Scanner) Scan() token.Token {
	return token.Token{}
}

// peek next byte in input. Return 0 on EOF.
func (s *Scanner) peek() byte {
	if s.offset+1 >= len(s.text) {
		return 0
	}
	return s.text[s.offset+1]
}

func (s *Scanner) consume() {
	s.ch = s.peek()
	s.offset++
}

func (s *Scanner) cur() byte {
	if s.eof() {
		return 0
	}
	return s.text[s.offset]
}

func (s *Scanner) eof() bool {
	return s.offset >= len(s.text)
}
