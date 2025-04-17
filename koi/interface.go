package koi

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/parser"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func ParseFile(filename string, src any) (*ast.Ast, error) {
	file := token.NewFile(filename, src)
	if file.Err != nil {
		return nil, file.Err
	}

	s := scanner.New(file)
	toks := s.ScanAll()

	if s.NumErrors > 0 {
		return nil, s.Error()
	}

	p := parser.New(file, toks)
	ast := p.Parse()

	return ast, p.Error()
}
