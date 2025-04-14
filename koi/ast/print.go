package ast

import (
	"fmt"
	"log"
	"strings"
)

type AstBuilder struct {
	strings.Builder
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
			a.WriteString("<nil statement>\n")

		default:
			log.Fatal("unknown top level statement type")
		}

		a.WriteByte('\n')
	}
}

func (a *AstBuilder) printFunc(node *Func) {
	a.WriteString(fmt.Sprintf("func %s", node.Name.Lexeme))
	a.printNamedTuple(node.Params)
	a.WriteByte(' ')
	a.printType(node.Type)
}

func (a *AstBuilder) printNamedTuple(node *NamedTuple) {
	if node == nil {
		a.WriteString("()")
		return
	}

	a.WriteByte('(')
	for i, f := range node.Fields {
		a.WriteString(f.Name.Lexeme)
		a.WriteByte(' ')
		a.printType(f.Type)

		if i != len(node.Fields)-1 {
			a.WriteString(", ")
		}
	}

	a.WriteByte(')')
}

func (a *AstBuilder) printType(node *Type) {
	if node == nil {
		a.WriteString("void")
		return
	}

	a.WriteString(node.T.Lexeme)
}
