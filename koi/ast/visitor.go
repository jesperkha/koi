package ast

import (
	"fmt"
	"strings"
)

type Visitor interface {
	VisitFunc(node *Func)
	VisitBlock(node *Block)
	VisitLiteral(node *Literal)
	VisitReturn(node *Return)
	VisitIdent(node *Ident)
	VisitCall(node *Call)
}

func (n *Func) Accept(v Visitor)    { v.VisitFunc(n) }
func (n *Block) Accept(v Visitor)   { v.VisitBlock(n) }
func (n *Literal) Accept(v Visitor) { v.VisitLiteral(n) }
func (n *Return) Accept(v Visitor)  { v.VisitReturn(n) }
func (n *Ident) Accept(v Visitor)   { v.VisitIdent(n) }
func (n *Call) Accept(v Visitor)    { v.VisitCall(n) }

// DebugVisitor implements the Visitor interface. It prints out each node as
// it visits it, forming a fully printed AST.
type DebugVisitor struct {
	sb     *strings.Builder
	indent int
}

func NewDebugVisitor() *DebugVisitor {
	return &DebugVisitor{
		sb:     &strings.Builder{},
		indent: 0,
	}
}

func (d *DebugVisitor) Print(tree *Ast) {
	tree.Walk(d)
	fmt.Println(d.String())
}

func (d *DebugVisitor) String() string {
	return d.sb.String()
}

func (d *DebugVisitor) write(f string, args ...any) {
	d.sb.WriteString(strings.Repeat("  ", d.indent) + fmt.Sprintf(f, args...) + "\n")
}

func (d *DebugVisitor) expr(e Expr) {
	d.indent++
	e.Accept(d)
	d.indent--
}

func (d *DebugVisitor) VisitBlock(node *Block) {
	d.write("block:")
	d.indent++

	for _, stmt := range node.Stmts {
		stmt.Accept(d)
	}

	d.indent--
}

func (d *DebugVisitor) VisitFunc(node *Func) {
	d.write("func: %s", node.Name.Lexeme)
	node.Block.Accept(d)
}

func (d *DebugVisitor) VisitReturn(node *Return) {
	d.write("return:")
	d.expr(node.E)
}

func (d *DebugVisitor) VisitLiteral(node *Literal) {
	d.write("literal: %s", node.Value)
}

func (d *DebugVisitor) VisitIdent(node *Ident) {
	d.write("ident: %s", node.Name)
}

func (d *DebugVisitor) VisitCall(node *Call) {
	d.write("call:")
	d.expr(node.Callee)

	for i, arg := range node.Args {
		d.write("$%d", i+1)
		d.expr(arg)
	}
}
