package koi

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/parser"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/types"
)

func ParseFile(filename string, src any) (*ast.Ast, *types.SemanticTable, error) {
	file := token.NewFile(filename, src)
	if file.Err != nil {
		return nil, nil, file.Err
	}

	s := scanner.New(file)
	toks := s.ScanAll()

	if s.NumErrors > 0 {
		return nil, nil, s.Error()
	}

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
