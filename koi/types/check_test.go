package types

import (
	"testing"

	"github.com/jesperkha/koi/koi/parser"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func tassert(t *testing.T, v bool, f string, args ...any) {
	if !v {
		t.Errorf(f, args...)
		t.FailNow()
	}
}

func checkerFrom(t *testing.T, src string) *Checker {
	file := token.NewFile("", src)
	tassert(t, file.Err == nil, "new file error: %s", file.Err)
	s := scanner.New(file)
	toks := s.ScanAll()
	tassert(t, s.NumErrors == 0, "scanAll error: %s", s.Error())
	p := parser.New(file, toks)
	tree := p.Parse()
	tassert(t, p.NumErrors == 0, "parse error: %s", p.Error())
	return NewChecker(file, tree)
}
