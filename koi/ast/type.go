package ast

import "github.com/jesperkha/koi/koi/token"

type TypeKind int

const (
	VOID TypeKind = iota
	STRING
	BYTE
	INT32
	FLOAT32
	ARRAY
)

type Type interface {
	Node

	// Get string representation of type, identical to the type syntax.
	String() string
}

type (
	PrimitiveType struct {
		T token.Token
	}

	ArrayType struct {
		LBrack token.Pos
		Type   Type
	}
)

func (p *PrimitiveType) String() string { return p.T.Lexeme }
func (p *PrimitiveType) Pos() token.Pos { return p.T.Pos }
func (p *PrimitiveType) End() token.Pos { return p.T.EndPos }

func (a *ArrayType) String() string { return "[]" + a.Type.String() }
func (a *ArrayType) Pos() token.Pos { return a.LBrack }
func (a *ArrayType) End() token.Pos { return a.Type.End() }
