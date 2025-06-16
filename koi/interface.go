package koi

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/ir"
	"github.com/jesperkha/koi/koi/parser"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/types"
)

func Tokenize(file *token.File) (tokens []token.Token, err error) {
	s := scanner.New(file)
	toks := s.ScanAll()

	if s.NumErrors > 0 {
		return tokens, s.Error()
	}

	return toks, nil
}

func Parse(file *token.File) (*ast.Ast, *types.SemanticTable, error) {
	toks, err := Tokenize(file)

	p := parser.New(file, toks)
	ast := p.Parse()

	if p.NumErrors > 0 {
		return nil, nil, p.Error()
	}

	c := types.NewChecker(file, ast)
	tbl, err := c.Check()
	if err != nil {
		return nil, nil, err
	}

	return ast, tbl, p.Error()
}

func GenerateIR(file *token.File) (i *ir.IR, err error) {
	tree, table, err := Parse(file)
	if err != nil {
		return nil, err
	}

	b := ir.NewBuilder(tree, table)
	return b.Build()
}
