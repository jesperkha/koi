package types

import (
	"fmt"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Checker struct {
	eh        util.ErrorHandler
	file      *token.File
	tree      *ast.Ast
	table     *SemanticTable
	NumErrors int
}

func NewChecker(file *token.File, tree *ast.Ast) *Checker {
	return &Checker{
		file:  file,
		tree:  tree,
		table: NewSemanticTable(),
	}
}

func (c *Checker) Check() (*SemanticTable, error) {
	assert(c.tree != nil, "tree is nil")
	c.tree.Walk(c)
	return c.table, c.Error()
}

func (c *Checker) Error() error {
	return c.eh.Error()
}

func assert(v bool, format string, args ...any) {
	if !v {
		panic(fmt.Sprintf("assertion failed: %s", fmt.Sprintf(format, args...)))
	}
}

func (c *Checker) err(node ast.Node, format string, arg ...any) {
	row := node.Pos().Row
	msg := fmt.Sprintf(format, arg...)
	c.eh.Pretty(row+1, c.file.Line(row), msg, node.Pos().Col, node.End().Col)
	c.NumErrors++
}

func (c *Checker) VisitFunc(node *ast.Func) {
	name := node.Name.Lexeme
	if name == "main" {
		c.visitMain(node)
		return
	}

	if f, ok := c.table.Symbol(name); ok {
		c.err(node, "function %s already declared on line %d", name, f.Pos.Row+1)
		return
	}

	retType, ok := c.visitType(node.RetType)
	if !ok {
		return
	}

	funcSymbol := Symbol{
		Name: node.Name.Lexeme,
		Kind: FuncSymbol,
		Pos:  node.Pos(),
		Type: retType,
	}

	c.table.Declare(funcSymbol) // Declare in global scope
	c.table.PushScope()         // Push function body
	c.table.SetReturnType(retType)

	// Declare all parameters as local variables
	for _, param := range node.Params.Fields {
		typ, ok := c.visitType(param.Type)
		if !ok {
			return
		}

		symbol := Symbol{
			Name: param.Name.Lexeme,
			Kind: VarSymbol,
			Pos:  param.Pos(),
			Type: typ,
		}

		c.table.Declare(symbol)
	}

	// Visit without scope because we just created one for the parameters.
	c.visitBlockWithoutScope(node.Block)

	if !c.table.HasReturned() && !TypeEquals(retType, voidType()) {
		c.err(node, "function never returns")
	}

	c.table.PopScope()
}

func (c *Checker) visitMain(node *ast.Func) {
	panic("visitMain not implemented")
}

func (c *Checker) visitType(node ast.Type) (typ Type, ok bool) {
	switch node := node.(type) {
	case *ast.PrimitiveType:
		return &PrimitiveType{kind: node.Kind}, true

	default:
		panic("unhandled type in visitType")
	}
}

// Same as VisitBlock, but does not create a new scope. This is because some
// statements like functions must declare symbols before entering the block,
// eg. parameter values.
func (c *Checker) visitBlockWithoutScope(node *ast.Block) {
	for _, stmt := range node.Stmts {
		stmt.Accept(c)
	}
}

func (c *Checker) VisitBlock(node *ast.Block) {
	c.table.PushScope()
	for _, stmt := range node.Stmts {
		stmt.Accept(c)
	}
	c.table.PopScope()
}

func (c *Checker) VisitReturn(node *ast.Return) {
	retType := c.table.ReturnType()
	c.table.MarkReturned()

	if node.E == nil {
		if !TypeEquals(voidType(), retType) {
			c.err(node, "expected return type %s", retType.String())
		}
	} else {
		t := c.evalExpr(node.E)
		if !TypeEquals(t, retType) {
			c.err(node.E, "expected return type %s, got %s", retType.String(), t.String())
		}
	}
}

func (c *Checker) VisitLiteral(node *ast.Literal) {
	// never called
}

// Evaluates given expression to a type and returns it. Returns nil on error.
func (c *Checker) evalExpr(node ast.Expr) Type {
	return c.evalLiteral(node.(*ast.Literal)) // no other types yet
}

func (c *Checker) evalLiteral(node *ast.Literal) Type {
	return &PrimitiveType{kind: ast.TokenToTypeKind(node.T.Type)}
}
