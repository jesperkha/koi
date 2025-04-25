package types

import "github.com/jesperkha/koi/koi/ast"

type Type interface {
}

func TypeEquals(a Type, b Type) bool {
	return false // TODO: type equals
}

// Void is commonly assigned as a placeholder or in cases where there is no
// type, therefore it is nice to have a helper to create a void type.
func voidType() TypeInfo {
	return TypeInfo{
		Name:       "void",
		Underlying: nil,
		Type:       &PrimitiveType{Kind: ast.VOID},
		Kind:       PrimitiveKind,
	}
}

type PrimitiveType struct {
	Kind ast.TypeKind
}
