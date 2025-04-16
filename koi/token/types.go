package token

import "fmt"

type TokenType int

const (
	ILLEGAL TokenType = iota
	EOF
	NEWLINE

	// Generic types
	STRING // String literal
	NUMBER // Integer literal
	IDENT  // Identifier

	// Keywords
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

	// Math
	PLUS    // +
	MINUS   // -
	STAR    // *
	SLASH   // /
	PERCENT // %

	// Logic
	EQ         // =
	EQ_EQ      // ==
	NOT_EQ     // !=
	PLUS_EQ    // +=
	MINUS_EQ   // -=
	MULT_EQ    // *=
	DIV_EQ     // /=
	GREATER    // >
	LESS       // <
	GREATER_EQ // >=
	LESS_EQ    // <=
	LPAREN     // (
	RPAREN     // )
	LBRACE     // {
	RBRACE     // }
	LBRACK     // [
	RBRACK     // ]
	AND        // &
	AND_AND    // &&
	OR         // |
	OR_OR      // ||
	NOT        // !

	// Other symbols
	DOT      // .
	COMMA    // ,
	SEMI     // ;
	COLON    // :
	COLON_EQ // :=

	// Types
	VOID
	INT
	FLOAT
	STRING_T
	BYTE
)

var tokenStrings = [...]string{
	ILLEGAL:    "illegal",
	EOF:        "eof",
	NEWLINE:    "newline",
	STRING:     "string",
	NUMBER:     "number",
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
	VOID:       "void",
	INT:        "int",
	FLOAT:      "float",
	STRING_T:   "string",
	BYTE:       "byte",
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
	"byte":    BYTE,
	"void":    VOID,
	"string":  STRING_T,
	"int":     INT,
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
