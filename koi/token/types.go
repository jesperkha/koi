package token

type TokenType int

const (
	ILLEGAL TokenType = iota
	EOF

	STRING
	INTEGER
	FLOAT
	IDENT

	TRUE
	FALSE
	RETURN
	FUNC
	IF
	ELSE
	FOR
	IMPORT
	PACKAGE
	NIL

	INT_TYPE
	UINT_TYPE
	FLOAT_TYPE
	STRING_TYPE
	BYTE_TYPE
)

var KeywordMap = map[string]TokenType{
	"true":    TRUE,
	"false":   FALSE,
	"return":  RETURN,
	"func":    FUNC,
	"if":      IF,
	"else":    ELSE,
	"for":     FOR,
	"import":  IMPORT,
	"package": PACKAGE,
	"nil":     NIL,
}
