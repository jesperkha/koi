package types

import (
	"fmt"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	errors    util.ErrorHandler
	file      *token.File
	ctx       *Context
	tree      *ast.Ast
	NumErrors int
}

func NewChecker(file *token.File, tree *ast.Ast) *Checker {
	return &Checker{
		ctx:  NewContext(),
		file: file,
		tree: tree,
	}
}

func (c *Checker) Check() *TypedAst {
	assert(c.tree != nil, "tree is nil")

	tree := &TypedAst{}
	for _, decl := range c.tree.Nodes {
		if d := c.visitDecl(decl); d != nil {
			tree.Nodes = append(tree.Nodes, d)
		}
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
		panic("unhandled declaration type in checker")
	}
}

func (c *Checker) visitStmt(node ast.Stmt) Stmt {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Return:
		return c.visitReturn(node)

	case *ast.Block:
		return c.visitBlock(node)

	default:
		panic("unhandled statement type in checker")
	}
}

func (c *Checker) visitExpr(node ast.Expr) Expr {
	assert(node != nil, "node is nil")

	switch node := node.(type) {
	case *ast.Literal:
		return c.visitLiteral(node)

	default:
		panic("unhandled statement type in checker")
	}
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
	c.ctx.Push()
	defer c.ctx.Pop()

	params := []*Field{}
	for i, param := range node.Params.Fields {
		assert(param != nil, "param idx=%d is nil", i)
		name := param.Name.Lexeme
		typ := c.visitType(param.Type)

		c.ctx.Set(name, typ)
		params = append(params, &Field{
			Name: name,
			Type: typ,
		})
	}

	retType := c.visitType(node.RetType)
	c.ctx.SetReturnType(retType)
	block := c.visitBlockNoScope(node.Block) // Scope is already made

	if !c.ctx.HasReturned() && !IsVoid(retType) {
		c.err(node, "function is missing return")
	}

	return &FuncDecl{
		Public:  node.Public,
		Name:    node.Name.Lexeme,
		RetType: retType,
		Params:  params,
		Block:   block,
	}
}

func (c *Checker) visitBlockNoScope(node *ast.Block) *Block {
	assert(node != nil, "node is nil")

	stmts := []Stmt{}
	for _, stmt := range node.Stmts {
		c.visitStmt(stmt)
	}

	return &Block{Stmts: stmts}
}

func (c *Checker) visitBlock(node *ast.Block) *Block {
	c.ctx.Push()
	defer c.ctx.Pop()
	return c.visitBlockNoScope(node)
}

func (c *Checker) visitReturn(node *ast.Return) *Return {
	assert(node != nil, "node is nil")
	r := c.ctx.GetReturnType()
	c.ctx.MarkReturned()

	if node.E == nil {
		t := &Primitive{Type: VOID}
		if !Equals(r, t) {
			c.err(node, "expected return type %s", r.String())
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
