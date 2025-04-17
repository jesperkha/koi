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

type PrimitiveType int

const (
	INT PrimitiveType = iota
	FLOAT
	STRING
	BYTE
	VOID
)

var tokenTypeToPrimitive = map[token.TokenType]PrimitiveType{
	token.VOID:     VOID,
	token.INT:      INT,
	token.FLOAT:    FLOAT,
	token.STRING_T: STRING,
	token.BYTE:     BYTE,
}

var primitiveToString = map[PrimitiveType]string{
	VOID:   "void",
	INT:    "int",
	FLOAT:  "float",
	STRING: "string",
	BYTE:   "byte",
}

type Primitive struct {
	Type PrimitiveType
}

func (p *Primitive) Underlying() Type {
	return p
}

func (p *Primitive) String() string {
	return primitiveToString[p.Type]
}
