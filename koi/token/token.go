package token

import "fmt"

type Token struct {
	Type   TokenType
	Pos    Pos    // Position of first character in token
	EndPos Pos    // Position of character immediately after token
	Lexeme string // The token as a string literal
	Length int    // The character length of the token

	// If the token is EOF. Always true if the type is EOF and
	// vice versa. Simply a shorthand for tok.Type == token.EOF.
	Eof bool

	// True if the token is malformed. This is different from TokenType.ILLEGAL
	// which is for unknown symbols. However, the Invalid field is always true
	// if the type is ILLEGAL.
	//
	// Example: the literal 1.2.3 will have the FLOAT type, but be Invalid
	// as it is malformed. This difference helps with error reporting.
	Invalid bool
}

func (t Token) String() string {
	return fmt.Sprintf("{%d '%s' c:%d r:%d}", t.Type, t.Lexeme, t.Pos.Col, t.Pos.Row)
}

type Pos struct {
	Col       int   // Column in file
	Row       int   // Row in file, same as line number -1
	Offset    int   // Byte offset in file
	File      *File // File this position refers to
	LineBegin int   // Offset of beginning of line token is on
}

type File struct {
	// Only the file name without path prefix.
	// Eg. foo/bar/faz.koi -> faz.koi
	Name string
}
