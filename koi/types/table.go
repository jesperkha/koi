package types

import "github.com/jesperkha/koi/koi/token"

// The SemanticTable includes all variable, function, and type declarations in
// the file, and their respective types.
type SemanticTable struct {
	globalScope  *Scope
	currentScope *Scope
	typeMap      map[string]Type // unused
}

// A Symbol is any declared name with a value. It can be a variable, constant,
// type, or function. Each symbol must have a corresponding type. In the case
// of function, the type is the return type.
type Symbol struct {
	Name     string     // Symbol name as it appears in the file.
	RefCount int        // How many times the symbol is referenced. 0 means unused.
	Exported bool       // If symbol is public. RefCount=0 is ok for exported symbols.
	Kind     SymbolKind // Type of symbol, eg. variable, function, etc.
	Pos      token.Pos
	Type     Type
}

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
		typeMap:      make(map[string]Type),
	}
}

// Symbol returns the Symbol value for the given name in the current scope, or
// any parent scopes. Returns ok bool to indicate if the symbol was found.
func (t *SemanticTable) Symbol(name string) (sym Symbol, ok bool) {
	return t.currentScope.Symbol(name)
}

// TypeOf gets the type info for the given name in the current scope, or any
// parent scopes. Returns ok bool to indicate if symbol was found.
func (t *SemanticTable) TypeOf(name string) (typ Type, ok bool) {
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

// Declare symbol in current scope, overriding any existing one.
func (t *SemanticTable) Declare(sym Symbol) {
	t.currentScope.Declare(sym)
}

func (t *SemanticTable) CurScope() *Scope {
	return t.currentScope
}

// Set return type for current scope, overriding any existing one.
func (t *SemanticTable) SetReturnType(typ Type) {
	t.currentScope.SetReturnType(typ)
}

// Get return type for current scope. Defaults to void type.
func (t *SemanticTable) ReturnType() Type {
	return t.currentScope.ReturnType()
}

// Mark current scope as having returned, making any succeeding statements
// unreachable.
func (t *SemanticTable) MarkReturned() {
	t.currentScope.MarkReturned()
}

// Reports whether the current scope has returned or not. Does not check any
// child scopes.
func (t *SemanticTable) HasReturned() bool {
	return t.currentScope.HasReturned()
}
