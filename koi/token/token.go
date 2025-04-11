package token

import "fmt"

type Token struct {
	Type    TokenType
	Pos     Pos    // Position of first character in token
	EndPos  Pos    // Position of character immediately after token
	Lexeme  string // The token as a string literal
	Length  int    // The character length of the token
	Invalid bool   // True if the token is an illegal token or malformed
	Eof     bool   // If the token is EOF
}

func (t Token) String() string {
	return fmt.Sprintf("{%d '%s' c:%d r:%d}", t.Type, t.Lexeme, t.Pos.Col, t.Pos.Row)
}

type Pos struct {
	Col    int   // Column in file
	Row    int   // Row in file, same as line number -1
	Offset int   // Byte offset in file
	File   *File // File this position refers to
}

type File struct {
	// Only the file name without path prefix.
	// Eg. foo/bar/faz.koi -> faz.koi
	Name string
}
