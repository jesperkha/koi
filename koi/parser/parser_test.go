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
	return New(&token.File{}, toks)
}

func TestNoInput(t *testing.T) {
	p := parserFrom("")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty input, got %s", p.Error())
	}
}

func TestEmptyFunction(t *testing.T) {
	p := parserFrom("pub func foo() {}\nfunc bar(a int) {}\nfunc faz(name string, age int) {}")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty function, got %s", p.Error())
	}
}
