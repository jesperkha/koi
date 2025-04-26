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

func TestEmptyFunction(t *testing.T) {
	c := checkerFrom(t, "func foo() void {}")
	if _, err := c.Check(); err != nil {
		t.Error(err)
	}
}

func TestLiteralReturn(t *testing.T) {
	cases := []string{
		"func f() int { return 123 }",
		"func f() bool { return true }",
		"func f() string { return \"hello\" }",
		"func f() float { return 12.0 }",
		"func f() void { return }",
		"func f(a int) int { return a }",
	}

	for _, cas := range cases {
		c := checkerFrom(t, cas)
		if _, err := c.Check(); err != nil {
			t.Error(err)
		}
	}
}

func TestIncorrectReturnType(t *testing.T) {
	cases := []string{
		"func f() int { return 1.23 }",
		"func f() bool { return 0 }",
		"func f() string { return 'a' }",
		"func f() int { }",
		"func f() void { return 0 }",
		"func f(a int) string { return a }",
	}

	for i, cas := range cases {
		c := checkerFrom(t, cas)
		if _, err := c.Check(); err == nil {
			t.Errorf("expected error from case %d", i+1)
		}
	}
}

func TestUndefinedIdent(t *testing.T) {
	cases := []string{
		"func f() int { return foo }",
	}

	for i, cas := range cases {
		c := checkerFrom(t, cas)
		if _, err := c.Check(); err == nil {
			t.Errorf("expected error from case %d", i+1)
		}
	}
}
