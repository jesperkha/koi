package types

import (
	"fmt"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	eh        *util.ErrorHandler
	file      *token.File
	tree      *ast.Ast
	table     *SemanticTable
	typ       Type
	NumErrors int
}

func NewChecker(file *token.File, tree *ast.Ast) *Checker {
	return &Checker{
		file:  file,
		tree:  tree,
		table: NewSemanticTable(),
		eh:    util.NewErrorHandler(),
	}
}

func (c *Checker) Check() (*SemanticTable, error) {
	util.Assert(c.tree != nil, "tree is nil")
	c.tree.Walk(c)
	return c.table, c.Error()
}

func (c *Checker) Error() error {
	return c.eh.Error()
}

func (c *Checker) err(node ast.Node, format string, arg ...any) {
	row := node.Pos().Row
	msg := fmt.Sprintf(format, arg...)
	c.eh.Pretty(row+1, c.file.Line(row), msg, node.Pos().Col, node.End().Col)
	c.NumErrors++
}

// Evaluate a node and return its type.
func (c *Checker) evaluate(node ast.Node) Type {
	node.Accept(c)
	return c.typ
}

// Set current contexts type
func (c *Checker) setType(t Type) {
	c.typ = t
}
