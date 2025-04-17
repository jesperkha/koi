package types

import (
	"fmt"
	"log"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	err   util.ErrorList
	scope *scope
}

func NewChecker() *Checker {
	return &Checker{
		scope: newScope(),
	}
}

func (c *Checker) Run(tree *ast.Ast) {
	assert(tree != nil, "tree is nil")

	for _, decl := range tree.Nodes {
		c.visitDecl(decl)
	}
}

func assert(v bool, format string, args ...any) {
	if !v {
		panic(fmt.Sprintf("assertion failed: %s", fmt.Sprintf(format, args...)))
	}
}

func (c *Checker) visitDecl(node ast.Decl) {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Func:
		c.visitFunc(node)

	default:
		log.Fatal("unhandled declaration type in checker")
	}
}

func (c *Checker) visitStmt(node ast.Stmt) {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Return:
		c.visitReturn(node)

	default:
		log.Fatal("unhandled statement type in checker")
	}
}

func (c *Checker) visitExpr(node ast.Expr) Type {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Literal:
		return c.visitLiteral(node)

	default:
		log.Fatal("unhandled statement type in checker")
	}

	return nil
}

func (c *Checker) visitType(node ast.Type) Type {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.PrimitiveType:
		if t, ok := tokenTypeToPrimitive[node.T.Type]; ok {
			return &Primitive{Type: t}
		}
		panic("unknown primitive type: " + node.String())

	default:
		panic("unhandled type")
	}
}

func (c *Checker) visitFunc(node *ast.Func) {
	c.scope.push()
	defer c.scope.pop()

	for i, param := range node.Params.Fields {
		assert(param != nil, "param idx=%d is nil", i)
		c.scope.set(param.Name.Lexeme, c.visitType(param.Type))
	}

	retType := c.visitType(node.RetType)
	c.scope.setReturnType(retType)

	for _, stmt := range node.Block.Stmts {
		c.visitStmt(stmt)
	}
}

func (c *Checker) visitReturn(node *ast.Return) {
	assert(node != nil, "node is ni")

	t := c.visitExpr(node.E)
	r := c.scope.getReturnType()
	if t.String() != r.String() {
		log.Printf("mismatched types %s and %s", t.String(), r.String())
	}
}

func (c *Checker) visitLiteral(node *ast.Literal) Type {
	assert(node != nil, "node is nil")
	t := node.T

	switch t.Type {
	case token.STRING:
		return &Primitive{Type: STRING}
	case token.NUMBER:
		if t.Float {
			return &Primitive{Type: FLOAT}
		}
		return &Primitive{Type: INT}
	}

	panic("invalid literal: " + node.T.String())
}
