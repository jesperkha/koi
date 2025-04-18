package types

import (
	"github.com/jesperkha/koi/koi/token"
)

type Type interface {
	// Underlying type. If the type is an alias for another, the this will
	// return the primitive or array type.
	Underlying() Type

	// Get string representation of type.
	String() string
}

func IsVoid(a Type) bool {
	return a.String() == "void" // Temp fix
}

func Equals(a Type, b Type) bool {
	return a.Underlying().String() == b.Underlying().String() // Temp fix
}

type PrimitiveType int

const (
	INT PrimitiveType = iota
	FLOAT
	STRING
	BYTE
	VOID
	BOOL
)

var tokenTypeToPrimitive = map[token.TokenType]PrimitiveType{
	token.VOID:     VOID,
	token.INT:      INT,
	token.FLOAT:    FLOAT,
	token.STRING_T: STRING,
	token.BYTE:     BYTE,
	token.BOOL:     BOOL,
}

var primitiveToString = map[PrimitiveType]string{
	VOID:   "void",
	INT:    "int",
	FLOAT:  "float",
	STRING: "string",
	BYTE:   "byte",
	BOOL:   "bool",
}

type Primitive struct {
	Type PrimitiveType
}

func (p *Primitive) Underlying() Type {
	if p.Type == BYTE {
		return &Primitive{Type: INT}
	}
	return p
}

func (p *Primitive) String() string {
	return primitiveToString[p.Type]
}
