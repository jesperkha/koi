package scanner

import "github.com/jesperkha/koi/koi/token"

type Scanner struct {
	file      *token.File
	text      []byte
	offset    int
	line      int
	numErrors int
}

// New makes a new Scanner object for the given file. The text is the raw text
// input to scan. Scanner only accepts ascii text.
func New(file *token.File, text []byte) *Scanner {
	return &Scanner{}
}

// Scan consumes the next token and returns it, advancing the Scanner.
func (s *Scanner) Scan() token.Token {
	return token.Token{}
}
