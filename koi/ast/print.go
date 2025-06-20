package ast

import (
	"fmt"
	"strings"
)

// DebugVisitor prints the AST identically to its source, with ideal formatting.
// Used for testing the parser (by comparing AST to string) and for debugging.
type DebugVisitor struct {
	sb          *strings.Builder
	indentLevel int
	tree        *Ast
	indented    bool
}

func NewDebugVisitor(tree *Ast) *DebugVisitor {
	return &DebugVisitor{
		sb:          &strings.Builder{},
		tree:        tree,
		indentLevel: 0,
	}
}

func (d *DebugVisitor) Print() {
	d.tree.Walk(d)
	fmt.Println(d.String())
}

func (d *DebugVisitor) String() string {
	return d.sb.String()
}

func (d *DebugVisitor) write(f string, args ...any) {
	if d.indentLevel != 0 && !d.indented {
		s := strings.Repeat("    ", d.indentLevel) + fmt.Sprintf(f, args...)
		d.sb.WriteString(s)
		d.indented = true
	} else {
		fmt.Fprintf(d.sb, f, args...)
	}
}

func (d *DebugVisitor) writeln(f string, args ...any) {
	d.write(f+"\n", args...)
	d.indented = false
}

func (d *DebugVisitor) indent(n Node) {
	d.indentLevel++
	n.Accept(d)
	d.indentLevel--
}

func (d *DebugVisitor) VisitBlock(node *Block) {
	d.writeln("{")
	for _, stmt := range node.Stmts {
		d.indent(stmt)
	}
	d.writeln("}")
}

func (d *DebugVisitor) VisitFunc(node *Func) {
	if node.Public {
		d.write("pub ")
	}

	d.write("func %s(", node.Name.Lexeme)
	for i, param := range node.Params.Fields {
		d.write("%s %s", param.Name.Lexeme, param.Type.String())
		if i < len(node.Params.Fields)-1 {
			d.write(", ")
		}
	}

	d.write(")")
	d.write(" %s ", node.RetType.String())
	node.Block.Accept(d)
}

func (d *DebugVisitor) VisitReturn(node *Return) {
	d.write("return ")
	node.E.Accept(d)
	d.writeln("")
}

func (d *DebugVisitor) VisitLiteral(node *Literal) {
	d.write("%s", node.Value)
}

func (d *DebugVisitor) VisitIdent(node *Ident) {
	d.write("%s", node.Name)
}

func (d *DebugVisitor) VisitCall(node *Call) {
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

func (d *DebugVisitor) VisitExprStmt(node *ExprStmt) {
	node.E.Accept(d)
	d.writeln("")
}
