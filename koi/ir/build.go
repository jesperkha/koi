package ir

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/types"
	"github.com/jesperkha/koi/koi/util"
)

// Builder implements the Visitor interface
type Builder struct {
	eh    *util.ErrorHandler
	tree  *ast.Ast
	table *types.SemanticTable
}

func NewBuilder(tree *ast.Ast, table *types.SemanticTable) *Builder {
	return &Builder{
		eh:    util.NewErrorHandler(),
		tree:  tree,
		table: table,
	}
}

func (b *Builder) Build() error {
	b.tree.Walk(b)
	return b.eh.Error()
}

func (b *Builder) VisitFunc(node *ast.Func) {
	node.Block.Accept(b)
}

func (b *Builder) VisitBlock(node *ast.Block) {
	for _, stmt := range node.Stmts {
		stmt.Accept(b)
	}
}

func (b *Builder) VisitReturn(node *ast.Return) {
	if node.E != nil {
		node.E.Accept(b)
	}
}

func (b *Builder) VisitLiteral(node *ast.Literal) {

}

func (b *Builder) VisitIdent(node *ast.Ident) {

}
