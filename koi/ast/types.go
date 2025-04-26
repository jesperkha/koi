package ast

import (
	"github.com/jesperkha/koi/koi/token"
)

type Type interface {
	// Get string representation of type, identical to the type syntax.
	String() string

	Pos() token.Pos
	End() token.Pos
}

type TypeKind int

const (
	INT TypeKind = iota
	FLOAT
	STRING
	BYTE
	VOID
	BOOL
)

type (
	PrimitiveType struct {
		Kind TypeKind
		T    token.Token
	}

	// ArrayType struct {
	// 	LBrack token.Pos
	// 	Type   Type
	// }
)

var typeKindNameMap = map[TypeKind]string{
	INT:    "int",
	FLOAT:  "float",
	STRING: "string",
	BYTE:   "byte",
	VOID:   "void",
	BOOL:   "bool",
}

func TypeKindToName(kind TypeKind) string {
	if name, ok := typeKindNameMap[kind]; ok {
		return name
	}
	panic("type kind without name")
}

var tokenToKindMap = map[token.TokenType]TypeKind{}

func TokenToTypeKind(typ token.TokenType) TypeKind {
	switch typ {
	case token.STRING, token.STRING_LIT:
		return STRING
	case token.BOOL, token.TRUE, token.FALSE:
		return BOOL
	case token.BYTE, token.BYTE_LIT:
		return BYTE
	case token.FLOAT, token.FLOAT_LIT:
		return FLOAT
	case token.INT, token.INT_LIT:
		return INT
	case token.VOID:
		return VOID
	}

	panic("token type without kind")
}

func (p *PrimitiveType) String() string { return p.T.Lexeme }
func (p *PrimitiveType) Pos() token.Pos { return p.T.Pos }
func (p *PrimitiveType) End() token.Pos { return p.T.EndPos }

// func (a *ArrayType) String() string { return "[]" + a.Type.String() }
// func (a *ArrayType) Pos() token.Pos { return a.LBrack }
// func (a *ArrayType) End() token.Pos { return a.Type.End() }
