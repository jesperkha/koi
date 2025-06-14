package types

type Scope struct {
	parent   *Scope   // Parent being nil means this is the global scope
	children []*Scope // List of all scopes (blocks) appearing in this one
	symbols  map[string]*Symbol

	// The current scopes expected return type, used for function bodies.
	// Defaults to void and is never nil. Child scopes will inherit this value.
	ret Type

	// True if there has been a return statement in the current scope (not
	// counting child scopes such as if-statements or other blocks).
	//
	// This value being true makes all succeeding statements in the current
	// scope unreachable.
	hasReturned bool
}

// Create new scope, inheriting relevant data from the parent. Parent may be
// nil (global scope), and in which case the scope will be marked as global
// and have default values.
func newScope(parent *Scope) *Scope {
	scope := &Scope{
		parent:  parent,
		ret:     voidType(),
		symbols: make(map[string]*Symbol),
	}

	if parent != nil {
		scope.ret = parent.ret
	}

	return scope
}

// Symbol returns the symbol mapped to name in this scope or any parent scope.
func (s *Scope) Symbol(name string) (sym *Symbol, ok bool) {
	if sym, ok := s.symbols[name]; ok {
		sym.RefCount++
		return sym, true
	}

	if s.parent != nil {
		return s.parent.Symbol(name)
	}

	return sym, false
}

// LocalSymbol returns the symbol mapped to name only in this scope.
func (s *Scope) LocalSymbol(name string) (sym *Symbol, ok bool) {
	sym, ok = s.symbols[name]
	return sym, ok
}

func (s *Scope) TypeOf(name string) (typ Type, ok bool) {
	return typ, ok
}

// Declare symbol in current scope, overriding any existing one.
func (s *Scope) Declare(sym *Symbol) {
	sym.Scope = s
	s.symbols[sym.Name] = sym
}

// Set return type for current scope, overriding any existing one.
func (s *Scope) SetReturnType(typ Type) {
	s.ret = typ
}

// Get return type for current scope. Defaults to void type.
func (s *Scope) ReturnType() Type {
	return s.ret
}

// Mark current scope as having returned, making any succeeding statements
// unreachable.
func (s *Scope) MarkReturned() {
	s.hasReturned = true
}

// Reports whether the current scope has returned or not. Does not check any
// child scopes.
func (s *Scope) HasReturned() bool {
	return s.hasReturned
}
