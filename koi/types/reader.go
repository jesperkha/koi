package types

import (
	"fmt"

	"github.com/jesperkha/koi/koi/ast"
)

// The TableReader hides a lot of unnecessary utility functions in the
// SemanticTable type and extends it with more suited methods for reading
// and fetching semantic data for building.
type TableReader interface {
	// Get symbol by name in either current or parent scope.
	Get(name string) *Symbol

	// Push new scope onto the reader.
	Push(block *ast.Block)

	// Pop current scope and return to parent.
	Pop()

	// Exported returns a list of all exported top-level symbols in the file.
	Exported() []*Symbol
}

func (t *SemanticTable) Get(name string) *Symbol {
	sym, ok := t.Symbol(name)
	if !ok {
		panic(fmt.Sprintf("undefined symbol after type check: '%s'", name))
	}
	return sym
}

func (t *SemanticTable) Push(block *ast.Block) {
	scope, ok := t.scopeMap[block]
	if !ok {
		panic("block with no assigned scope")
	}
	t.currentScope = scope
}

func (t *SemanticTable) Pop() {
	t.currentScope = t.currentScope.parent
}

func (t *SemanticTable) Exported() []*Symbol {
	exported := []*Symbol{}
	for _, v := range t.globalScope.symbols {
		if v.Exported {
			exported = append(exported, v)
		}
	}

	return exported
}
