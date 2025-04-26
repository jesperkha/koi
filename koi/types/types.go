package types

import "github.com/jesperkha/koi/koi/ast"

type Type interface {
	Underlying() Type
	String() string
}

func TypeEquals(a Type, b Type) bool {
	return a.String() == b.String() // temp fix
}

// Void is commonly assigned as a placeholder or in cases where there is no
// type, therefore it is nice to have a helper to create a void type.
func voidType() Type {
	return &PrimitiveType{kind: ast.VOID}
}

type PrimitiveType struct {
	kind ast.TypeKind
}

func (p *PrimitiveType) Underlying() Type { return p }
func (p *PrimitiveType) String() string   { return ast.TypeKindToName(p.kind) }
