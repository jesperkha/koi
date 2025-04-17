package ast

import (
	"fmt"
	"log"
	"strings"
)

type AstBuilder struct {
	strings.Builder
	depth int
}

func Print(tree *Ast) {
	b := &AstBuilder{}
	b.Build(tree)
	fmt.Println(b.String())
}

func (a *AstBuilder) Build(tree *Ast) {
	for _, stmt := range tree.Nodes {
		switch stmt := stmt.(type) {
		case *Func:
			a.printFunc(stmt)
		case nil:
			a.writeln("<nil statement>")
		default:
			log.Fatal("[ERR: unknown top level statement type]")
		}

		a.write("\n")
	}
}

func (a *AstBuilder) write(s string) {
	a.WriteString(strings.Repeat("    ", a.depth) + s)
}

func (a *AstBuilder) writeln(s string) {
	a.write(s + "\n")
}

func (a *AstBuilder) printFunc(node *Func) {
	a.write(fmt.Sprintf("func %s", node.Name.Lexeme))
	a.printNamedTuple(node.Params)
	a.write(" ")
	a.printType(node.RetType)
	a.write("\n")
	a.printBlock(node.Block)
}

func (a *AstBuilder) printNamedTuple(node *NamedTuple) {
	if node == nil {
		a.write("()")
		return
	}

	a.write("(")
	for i, f := range node.Fields {
		a.write(f.Name.Lexeme)
		a.write(" ")
		a.printType(f.Type)

		if i != len(node.Fields)-1 {
			a.write(", ")
		}
	}

	a.write(")")
}

func (a *AstBuilder) printType(node Type) {
	if node == nil {
		a.write("<no-type>")
		return
	}

	a.write(node.String())
}

func (a *AstBuilder) printStmt(node Stmt) {
	switch node := node.(type) {
	case *Return:
		a.printReturn(node)
	case *Block:
		a.printBlock(node)
	case nil:
		a.writeln("<nil>")

	default:
		a.writeln("[ERR: unhandled statement type]")
	}
}

func (a *AstBuilder) printBlock(node *Block) {
	a.writeln("{")
	a.depth++
	defer func() {
		a.depth--
		a.writeln("}")
	}()

	if node.Empty {
		a.writeln("<empty>")
		return
	}

	for _, s := range node.Stmts {
		a.printStmt(s)
	}
}

func (a *AstBuilder) printReturn(node *Return) {
	a.writeln("return")
	a.depth++
	a.printExpr(node.E)
	a.depth--
}

func (a *AstBuilder) printExpr(node Expr) {
	switch node := node.(type) {
	case *Literal:
		a.printLiteral(node)
	case nil:
		a.writeln("<empty>")
	default:
		a.writeln("[ERR: unhandled expression type]")
	}
}

func (a *AstBuilder) printLiteral(node *Literal) {
	a.writeln(fmt.Sprintf("literal: %s", node.Value))
}
