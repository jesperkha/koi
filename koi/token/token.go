package token

type Token struct {
	Pos     Pos    // Position of first character in token
	EndPos  Pos    // Position of last character in token
	Lexeme  string // The token as a string literal
	Length  int    // The character length of the token
	Invalid bool   // True if the token is an illegal token or malformed
	Eof     bool   // If the token is EOF
}

type Pos struct {
	Col  int   // Column in file
	Row  int   // Row in file, same as line number -1
	File *File // File this position refers to
}

type File struct {
	// Only the file name without path prefix.
	// Eg. foo/bar/faz.koi -> faz.koi
	Name string
}
