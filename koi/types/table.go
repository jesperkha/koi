package types

import "github.com/jesperkha/koi/koi/token"

// The SemanticTable includes all variable, function, and type declarations in
// the file, and their respective types.
type SemanticTable struct {
	globalScope  *Scope
	currentScope *Scope
	typeMap      map[string]TypeInfo
}

// A Symbol is any declared name with a value. It can be a variable, constant,
// type, or function. Each symbol must have a corresponding type. In the case
// of function, the type is the return type.
type Symbol struct {
	Name  string
	Kind  SymbolKind
	Pos   token.Pos
	Type  TypeInfo
	Scope *Scope
}

// TypeInfo describes the type of a symbol. Underlying points to the base type
// and is nil in most cases.
type TypeInfo struct {
	Name       string    // Raw name, eg. "int", "[]string" etc
	Kind       TypeKind  // Type kind is a general type, eg. primitive or array
	Type       Type      // The actual type, in the case of aliasing, this is nil
	Underlying *TypeInfo // For aliases, points to underlying type
}

type TypeKind int

const (
	PrimitiveType TypeKind = iota
	ArrayType
)

type SymbolKind int

const (
	VarSymbol SymbolKind = iota
	ConstSymbol
	FuncSymbol
	TypeSymbol
)

func NewSemanticTable() *SemanticTable {
	global := newScope(nil)
	return &SemanticTable{
		globalScope:  global,
		currentScope: global,
	}
}

// Symbol returns the Symbol value for the given name in the current scope, or
// any parent scopes. Returns ok bool to indicate if the symbol was found.
func (t *SemanticTable) Symbol(name string) (sym Symbol, ok bool) {
	return t.currentScope.Symbol(name)
}

// TypeOf gets the type info for the given name in the current scope, or any
// parent scopes. Returns ok bool to indicate if symbol was found.
func (t *SemanticTable) TypeOf(name string) (typ TypeInfo, ok bool) {
	return t.currentScope.TypeOf(name)
}

// Push new scope, making it the child of the current one.
func (t *SemanticTable) PushScope() {
	scope := newScope(t.currentScope)
	t.currentScope.children = append(t.currentScope.children, scope)
	t.currentScope = scope
}

// Pop current scope, returning to its parent.
func (t *SemanticTable) PopScope() {
	t.currentScope = t.currentScope.parent
}
