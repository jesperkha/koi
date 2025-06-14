package types

import "fmt"

// The TableReader hides a lot of unnecessary utility functions in the
// SemanticTable type and extends it with more suited methods for reading
// and fetching semantic data for building.
type TableReader interface {
	// Get symbol by name in either current or parent scope.
	Get(name string) *Symbol

	// Push new scope onto the reader.
	Push(scope *Scope)

	// Pop current scope and return to parent.
	Pop()
}

func (st *SemanticTable) Get(name string) *Symbol {
	sym, ok := st.Symbol(name)
	if !ok {
		panic(fmt.Sprintf("undefined symbol after type check: '%s'", name))
	}
	return sym
}

func (st *SemanticTable) Push(scope *Scope) {
	st.currentScope = scope
}

func (st *SemanticTable) Pop() {
	st.currentScope = st.currentScope.parent
}
