package types

import (
	"fmt"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	errors util.ErrorHandler
	file   *token.File
	// ctx       *Context
	tree      *ast.Ast
	NumErrors int
}

func NewChecker(file *token.File, tree *ast.Ast) *Checker {
	return &Checker{
		// ctx:  NewContext(),
		file: file,
		tree: tree,
	}
}

func (c *Checker) Check() {
	assert(c.tree != nil, "tree is nil")
	c.tree.Walk(c)
}

func (c *Checker) Error() error {
	return c.errors.Error()
}

func assert(v bool, format string, args ...any) {
	if !v {
		panic(fmt.Sprintf("assertion failed: %s", fmt.Sprintf(format, args...)))
	}
}

func (c *Checker) err(node ast.Node, format string, arg ...any) {
	row := node.Pos().Row
	msg := fmt.Sprintf(format, arg...)
	c.errors.Pretty(row+1, c.file.Line(row), msg, node.Pos().Col, node.End().Col)
	c.NumErrors++
}

func (c *Checker) VisitBlock(node *ast.Block) {
	for _, stmt := range node.Stmts {
		stmt.Accept(c)
	}
}

func (c *Checker) VisitFunc(node *ast.Func) {
	node.Block.Accept(c)
}

func (c *Checker) VisitReturn(node *ast.Return) {
	node.E.Accept(c)
}

func (c *Checker) VisitLiteral(node *ast.Literal) {

}
