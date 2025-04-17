package types

import (
	"fmt"
	"log"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	errors    util.ErrorHandler
	file      *token.File
	scope     *scope
	tree      *ast.Ast
	NumErrors int
}

func NewChecker(file *token.File, tree *ast.Ast) *Checker {
	return &Checker{
		scope: newScope(),
		file:  file,
		tree:  tree,
	}
}

func (c *Checker) Run() {
	assert(c.tree != nil, "tree is nil")

	for _, decl := range c.tree.Nodes {
		c.visitDecl(decl)
	}
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
	if !Equals(t, r) {
		c.err(node, "type %s does not match expected return type %s", t.String(), r.String())
	}
}

func (c *Checker) visitLiteral(node *ast.Literal) Type {
	assert(node != nil, "node is nil")
	t := node.T

	switch t.Type {
	case token.TRUE, token.FALSE:
		return &Primitive{Type: BOOL}

	case token.STRING:
		return &Primitive{Type: STRING}

	case token.NUMBER:
		if t.Float {
			return &Primitive{Type: FLOAT}
		}
		return &Primitive{Type: INT}

	case token.BYTE_STR:
		return &Primitive{Type: BYTE}
	}

	panic("unhandled literal: " + node.T.String())
}
