package token

import "fmt"

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
	PUB

	PLUS
	MINUS
	STAR
	SLASH
	DOT
	COMMA
	SEMI
	COLON
	COLON_EQ
	EQ
	EQ_EQ
	NOT_EQ
	PLUS_EQ
	MINUS_EQ
	MULT_EQ
	DIV_EQ
	GREATER
	LESS
	GREATER_EQ
	LESS_EQ
	LPAREN
	RPAREN
	LBRACE
	RBRACE
	LBRACK
	RBRACK
	AND
	AND_AND
	OR
	OR_OR
	PERCENT
	NOT
)

var tokenStrings = [...]string{
	ILLEGAL:    "illegal",
	EOF:        "eof",
	STRING:     "string",
	INTEGER:    "int",
	FLOAT:      "float",
	IDENT:      "identifier",
	TRUE:       "true",
	FALSE:      "false",
	RETURN:     "return",
	FUNC:       "func",
	IF:         "if",
	ELSE:       "else",
	FOR:        "for",
	IMPORT:     "import",
	PACKAGE:    "package",
	NIL:        "nil",
	PUB:        "pub",
	PLUS:       "+",
	MINUS:      "-",
	STAR:       "*",
	SLASH:      "/",
	DOT:        ".",
	COMMA:      ",",
	SEMI:       ";",
	COLON:      ":",
	COLON_EQ:   ":=",
	EQ:         "=",
	EQ_EQ:      "==",
	NOT_EQ:     "!=",
	PLUS_EQ:    "+=",
	MINUS_EQ:   "-=",
	MULT_EQ:    "*=",
	DIV_EQ:     "/=",
	GREATER:    ">",
	LESS:       "<",
	GREATER_EQ: ">=",
	LESS_EQ:    "<=",
	LPAREN:     "(",
	RPAREN:     ")",
	LBRACE:     "{",
	RBRACE:     "}",
	LBRACK:     "[",
	RBRACK:     "]",
	AND:        "&",
	AND_AND:    "&&",
	OR:         "|",
	OR_OR:      "||",
	PERCENT:    "%",
	NOT:        "!",
}

func String(t TokenType) string {
	if int(t) >= len(tokenStrings) {
		panic(fmt.Sprintf("token type with no string: %d", t))
	}

	return tokenStrings[t]
}

var Keywords = map[string]TokenType{
	"pub":     PUB,
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

var SingleSymbols = map[string]TokenType{
	"+": PLUS,
	"-": MINUS,
	"*": STAR,
	"/": SLASH,
	".": DOT,
	",": COMMA,
	";": SEMI,
	":": COLON,
	"=": EQ,
	">": GREATER,
	"<": LESS,
	"(": LPAREN,
	")": RPAREN,
	"{": LBRACE,
	"}": RBRACE,
	"[": LBRACK,
	"]": RBRACK,
	"&": AND,
	"|": OR,
	"%": PERCENT,
	"!": NOT,
}

var DoubleSymbols = map[string]TokenType{
	"||": OR_OR,
	">=": GREATER_EQ,
	"<=": LESS_EQ,
	"&&": AND_AND,
	"==": EQ_EQ,
	"!=": NOT_EQ,
	"+=": PLUS_EQ,
	"-=": MINUS_EQ,
	"*=": MULT_EQ,
	"/=": DIV_EQ,
	":=": COLON_EQ,
}
