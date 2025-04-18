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

func TestValidReturnType(t *testing.T) {
	cases := []string{
		"func f() void { return }",
		"func f() int { return 0 }",
		"func f() float { return 1.0 }",
		"func f() byte { return 'a' }",
		"func f() byte { return 10 }",
		"func f() int { return 'a' }",
		"func f() bool { return false }",
		"func f() bool { return true }",
		"func f() string { return \"hello\" }",
	}

	for i, cas := range cases {
		c := checkerFrom(t, cas)
		tassert(t, c.Check() != nil, "case %d: expected non-nil ast", i+1)
		tassert(t, c.NumErrors == 0, "case %d: expected no errors, got %s", i+1, c.Error())
	}
}

func TestInvalidReturnType(t *testing.T) {
	cases := []string{
		"func f() void { return 0 }",
		"func f() float { return 1 }",
		"func f() int { return 1.0 }",
		"func f() bool { return 1 }",
		"func f() byte { return \"hello\" }",
		"func f() int { return }",
	}

	for i, cas := range cases {
		c := checkerFrom(t, cas)
		tassert(t, c.Check() != nil, "case %d: expected non-nil ast", i+1)
		tassert(t, c.NumErrors != 0, "case %d: expected error, got none", i+1)
	}
}
