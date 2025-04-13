package koi

import (
	"fmt"
	"os"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/parser"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func ParseFile(filename string, src any) (*ast.Ast, error) {
	file := &token.File{Name: filename}

	srcBytes, err := readSource(filename, src)
	if err != nil {
		return nil, err
	}

	s := scanner.New(file, srcBytes)
	toks := s.ScanAll()

	if s.NumErrors > 0 {
		return nil, s.Error()
	}

	p := parser.New(file, toks)
	ast := p.Parse()

	return ast, p.Error()
}

func readSource(filename string, src any) ([]byte, error) {
	if src != nil {
		switch src := src.(type) {
		case string:
			return []byte(src), nil

		case []byte:
			return src, nil

		default:
			return nil, fmt.Errorf("invalid src type")
		}
	}

	return os.ReadFile(filename)
}
