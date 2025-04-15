package parser

import (
	"testing"

	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func parserFrom(src string) *Parser {
	file := &token.File{}
	s := scanner.New(file, []byte(src))
	toks := s.ScanAll()
	return New(&token.File{}, toks, []byte(src))
}

func TestNoInput(t *testing.T) {
	p := parserFrom("")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty input, got %s", p.Error())
	}
}

func TestEmptyFunction(t *testing.T) {
	p := parserFrom("pub func foo() void {}\nfunc bar(a int) void {}\nfunc faz(name string, age int) void {}")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty function, got %s", p.Error())
	}
}

func assert(t *testing.T, expr bool, msg string) {
	if !expr {
		t.Errorf("assert failed: %s", msg)
	}
}

func TestLiteral(t *testing.T) {
	p := parserFrom("123 1.23 true false nil \"hello\"")
	for i := 0; i < 6; i++ {
		assert(t, p.parseLiteral() != nil, "expected not nil")
	}
}
