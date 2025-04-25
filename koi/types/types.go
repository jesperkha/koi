package types

type Type interface {
}

// Void is commonly assigned as a placeholder or in cases where there is no
// type, therefore it is nice to have a helper to create a void type.
func voidType() TypeInfo {
	return TypeInfo{
		Name:       "void",
		Underlying: nil,
		Type:       nil,
		Kind:       PrimitiveType,
	}
}
