package ir

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/types"
	"github.com/jesperkha/koi/koi/util"
)

// Builder implements the Visitor interface
type Builder struct {
	eh   *util.ErrorHandler
	tree *ast.Ast
	tr   types.TableReader
	ir   []Instruction
}

func NewBuilder(tree *ast.Ast, reader types.TableReader) *Builder {
	return &Builder{
		eh:   util.NewErrorHandler(),
		tree: tree,
		tr:   reader,
	}
}

func (b *Builder) Build() ([]Instruction, error) {
	b.tree.Walk(b)
	return b.ir, b.eh.Error()
}

func (b *Builder) VisitFunc(node *ast.Func) {
	funcName := node.Name.Lexeme

	b.ir = append(b.ir, Instruction{
		Op:      FUNC,
		Name:    funcName,
		Public:  node.Public,
		RetType: b.tr.Get(funcName).Type,
	})

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
