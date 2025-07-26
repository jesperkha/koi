package ast

import (
	"fmt"
	"strings"
)

// Get a string representation of the AST identical to its source, with ideal formatting.
// Used for testing the parser (by comparing AST to string) and for debugging.
func String(tree *Ast) string {
	return newDebugVisitor(tree).String()
}

func Print(tree *Ast) {
	fmt.Println(String(tree))
}

type debugVisitor struct {
	sb          *strings.Builder
	indentLevel int
	tree        *Ast
	indented    bool
}

func newDebugVisitor(tree *Ast) *debugVisitor {
	return &debugVisitor{
		sb:          &strings.Builder{},
		tree:        tree,
		indentLevel: 0,
	}
}

func (d *debugVisitor) String() string {
	d.tree.Walk(d)
	return d.sb.String()
}

func (d *debugVisitor) write(f string, args ...any) {
	if d.indentLevel != 0 && !d.indented {
		s := strings.Repeat("    ", d.indentLevel) + fmt.Sprintf(f, args...)
		d.sb.WriteString(s)
		d.indented = true
	} else {
		fmt.Fprintf(d.sb, f, args...)
	}
}

func (d *debugVisitor) writeln(f string, args ...any) {
	d.write(f+"\n", args...)
	d.indented = false
}

func (d *debugVisitor) indent(n Node) {
	d.indentLevel++
	n.Accept(d)
	d.indentLevel--
}

func (d *debugVisitor) VisitBlock(node *Block) {
	if node == nil {
		return
	}
	d.writeln("{")
	for _, stmt := range node.Stmts {
		d.indent(stmt)
	}
	d.writeln("}")
}

func (d *debugVisitor) VisitFunc(node *Func) {
	if node == nil {
		return
	}
	if node.Public {
		d.write("pub ")
	}

	d.write("func %s(", node.Name.Lexeme)
	for i, param := range node.Params.Fields {
		d.write("%s ", param.Name.Lexeme)
		param.Type.Accept(d)

		// d.write("%s %s", param.Name.Lexeme, param.Type.String())
		if i < len(node.Params.Fields)-1 {
			d.write(", ")
		}
	}

	d.write(")")
	d.write(" %s ", node.RetType.String())
	node.Block.Accept(d)
}

func (d *debugVisitor) VisitReturn(node *Return) {
	if node == nil {
		return
	}
	d.write("return ")
	if node.E != nil {
		node.E.Accept(d)
	}
	d.writeln("")
}

func (d *debugVisitor) VisitLiteral(node *Literal) {
	if node == nil {
		return
	}
	d.write("%s", node.Value)
}

func (d *debugVisitor) VisitIdent(node *Ident) {
	if node == nil {
		return
	}
	d.write("%s", node.Name)
}

func (d *debugVisitor) VisitCall(node *Call) {
	if node == nil {
		return
	}
	node.Callee.Accept(d)
	d.write("(")
	for i, arg := range node.Args {
		arg.Accept(d)
		if i < len(node.Args)-1 {
			d.write(", ")
		}
	}
	d.write(")")
}

func (d *debugVisitor) VisitExprStmt(node *ExprStmt) {
	if node == nil {
		return
	}
	node.E.Accept(d)
	d.writeln("")
}

func (d *debugVisitor) VisitPrimitiveType(node *PrimitiveType) {
	d.write("%s", node.String())
}
