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

func (c *Checker) Check() *TypedAst {
	assert(c.tree != nil, "tree is nil")

	tree := &TypedAst{}
	for _, decl := range c.tree.Nodes {
		c.visitDecl(decl)
	}

	return tree
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

func (c *Checker) visitDecl(node ast.Decl) Decl {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Func:
		return c.visitFunc(node)

	default:
		log.Fatal("unhandled declaration type in checker")
	}

	// Unreachable
	return nil
}

func (c *Checker) visitStmt(node ast.Stmt) Stmt {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Return:
		return c.visitReturn(node)

	default:
		log.Fatal("unhandled statement type in checker")
	}

	// Unreachable
	return nil
}

func (c *Checker) visitExpr(node ast.Expr) Expr {
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

func (c *Checker) visitFunc(node *ast.Func) *FuncDecl {
	c.scope.push()
	defer c.scope.pop()

	params := []*Field{}
	for i, param := range node.Params.Fields {
		assert(param != nil, "param idx=%d is nil", i)
		name := param.Name.Lexeme
		typ := c.visitType(param.Type)

		c.scope.set(name, typ)
		params = append(params, &Field{
			Name: name,
			Type: typ,
		})
	}

	retType := c.visitType(node.RetType)
	c.scope.setReturnType(retType)

	for _, stmt := range node.Block.Stmts {
		c.visitStmt(stmt)
	}

	return &FuncDecl{
		Public:  node.Public,
		Name:    node.Name.Lexeme,
		RetType: retType,
		Params:  params,
	}
}

func (c *Checker) visitReturn(node *ast.Return) *Return {
	assert(node != nil, "node is ni")
	r := c.scope.getReturnType()

	if node.E == nil {
		t := &Primitive{Type: VOID}
		if !Equals(r, t) {
			c.err(node, "type %s does not match expected return type %s", t.String(), r.String())
		}

		return &Return{E: nil}
	}

	e := c.visitExpr(node.E)
	if !Equals(e.Type(), r) {
		c.err(node, "type %s does not match expected return type %s", e.Type().String(), r.String())
	}

	return &Return{E: e}
}

func (c *Checker) visitLiteral(node *ast.Literal) *Literal {
	assert(node != nil, "node is nil")
	t := node.T

	var typ Type
	expr := &Literal{
		TokenType: t.Type,
		Value:     t.Lexeme,
	}

	switch t.Type {
	case token.TRUE, token.FALSE:
		typ = &Primitive{Type: BOOL}

	case token.STRING:
		typ = &Primitive{Type: STRING}

	case token.NUMBER:
		if t.Float {
			typ = &Primitive{Type: FLOAT}
		} else {
			typ = &Primitive{Type: INT}
		}

	case token.BYTE_STR:
		typ = &Primitive{Type: BYTE}

	default:
		panic("unhandled literal: " + node.T.String())
	}

	expr.LitType = typ
	return expr
}
