package ast

import "strings"

type Visitor interface {
	VisitFunc(node *Func)
	VisitBlock(node *Block)
	VisitLiteral(node *Literal)
	VisitReturn(node *Return)
	VisitIdent(node *Ident)
}

func (n *Func) Accept(v Visitor)    { v.VisitFunc(n) }
func (n *Block) Accept(v Visitor)   { v.VisitBlock(n) }
func (n *Literal) Accept(v Visitor) { v.VisitLiteral(n) }
func (n *Return) Accept(v Visitor)  { v.VisitReturn(n) }
func (n *Ident) Accept(v Visitor)   { v.VisitIdent(n) }

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

func (d *DebugVisitor) String() string {
	return d.sb.String()
}

func (d *DebugVisitor) write(s string) {
	d.sb.WriteString(strings.Repeat("  ", d.indent) + s + "\n")
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
	d.write("func: " + node.Name.Lexeme)
	node.Block.Accept(d)
}

func (d *DebugVisitor) VisitReturn(node *Return) {
	d.write("return:")
	d.indent++
	node.E.Accept(d)
	d.indent--
}

func (d *DebugVisitor) VisitLiteral(node *Literal) {
	d.write("literal: " + node.Value)
}

func (d *DebugVisitor) VisitIdent(node *Ident) {
	d.write("ident: " + node.Name)
}
