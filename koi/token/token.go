package token

import "fmt"

type Token struct {
	Type   TokenType
	Pos    Pos    // Position of first character in token
	EndPos Pos    // Position of character immediately after token
	Lexeme string // The token as a string literal
	Float  bool   // If Type is NUMBER, this is true for floating point literals

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
	return fmt.Sprintf("{%s '%s' c:%d r:%d}", tokenStrings[t.Type], t.Lexeme, t.Pos.Col, t.Pos.Row)
}

// Print tokens to standard out as formatted by Token.String().
func Print(toks []Token) {
	for _, t := range toks {
		fmt.Println(t.String())
	}
}

type Pos struct {
	Col       int   // Column in file
	Row       int   // Row in file, same as line number -1
	Offset    int   // Byte offset in file
	File      *File // File this position refers to
	LineBegin int   // Offset of beginning of line token is on
}
