package parser

import (
	"strings"
	"testing"

	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func parserFrom(src string) *Parser {
	file := token.NewFile("test", src)
	s := scanner.New(file)
	toks := s.ScanAll()
	return New(file, toks)
}

func TestNoInput(t *testing.T) {
	p := parserFrom("")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty input, got %s", p.Error())
	}
}

func TestEmptyFunction(t *testing.T) {
	p := parserFrom("pub func foo() void {} func bar(a int) void {} func faz(name string, age int) void {}")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty function, got %s", p.Error())
	}
}

func TestFunctionWithReturn(t *testing.T) {
	p := parserFrom("func f(a int, b string) int { return 0 }")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error, got %s", p.Error())
	}
}

func assert(t *testing.T, expr bool, msg string) {
	if !expr {
		t.Errorf("assert failed: %s", msg)
	}
}

func TestLiteral(t *testing.T) {
	p := parserFrom("123 1.23 true false nil \"hello\"")
	for range 6 {
		assert(t, p.parseLiteral() != nil, "expected not nil")
	}
}

func TestPrimitiveTypes(t *testing.T) {
	src := "int float []int [][]string bool byte void"
	p := parserFrom(src)
	expect := strings.SplitSeq(src, " ")

	for s := range expect {
		got := p.parseType()

		if p.NumErrors != 0 {
			t.Errorf("expected no error, got %s", p.errors.Error())
			t.FailNow()
		}

		if got == nil {
			t.Errorf("expected %s, got <nil>", s)
		}

		if got.String() != s {
			t.Errorf("expected %s, got %s", s, got.String())
		}
	}
}
