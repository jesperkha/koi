package parser

import (
	"strings"
	"testing"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func parserFrom(src string) *Parser {
	file := token.NewFile("test", src)
	s := scanner.New(file)
	toks := s.ScanAll()
	return New(file, toks)
}

func parseAndCompare(t *testing.T, src string) {
	p := parserFrom(src)
	tree := p.Parse()
	if p.Error() != nil {
		t.Fatal(p.Error())
	}

	expectLines := strings.Split(strings.TrimSpace(src), "\n")
	printLines := strings.Split(strings.TrimSpace(ast.String(tree)), "\n")

	if len(printLines) != len(expectLines) {
		t.Fatal("number of lines in expected and actual do not match")
	}

	for i, line := range printLines {
		expect := strings.TrimSpace(expectLines[i])
		got := strings.TrimSpace(line)
		if got != expect {
			t.Fatalf("expected equal, line %d\n\texpect '%s'\n\tgot '%s'", i+1, expect, got)
		}
	}
}

func assert(t *testing.T, expr bool, msg string) {
	if !expr {
		t.Errorf("assert failed: %s", msg)
	}
}

func TestNoInput(t *testing.T) {
	p := parserFrom("")
	p.Parse()

	if p.Error() != nil {
		t.Errorf("expected no error for empty input, got %s", p.Error())
	}
}

func TestEmptyFunction(t *testing.T) {
	parseAndCompare(t, `
		pub func foo() void {
		}
	`)

	parseAndCompare(t, `
		func bar(a int) void {
		}
	`)

	parseAndCompare(t, `
		func faz(name string, age int) void {
		}
	`)
}

func TestFunctionWithReturn(t *testing.T) {
	parseAndCompare(t, `
		func foo(a int, b float) int {
			return a
		}
	`)

	parseAndCompare(t, `
		pub func bar() void {
			return
		}
	`)
}

func TestLiteral(t *testing.T) {
	p := parserFrom("123 1.23 true false nil \"hello\"")
	for range 6 {
		assert(t, p.parseLiteral() != nil, "expected not nil")
	}
}

func TestPrimitiveTypes(t *testing.T) {
	src := "int float bool byte void"
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

func TestCall(t *testing.T) {
	parseAndCompare(t, `
		func foo() void {
			bar()
		}
	`)

	parseAndCompare(t, `
		func foo() void {
			bar(1, "hello", a)
		}
	`)

	parseAndCompare(t, `
		func foo() void {
			chained(1)(2)
		}
	`)
}
