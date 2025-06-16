package ir_test

import (
	"fmt"
	"strings"
	"testing"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ir"
	"github.com/jesperkha/koi/koi/token"
)

func irFrom(t *testing.T, src string) []ir.Instruction {
	file := token.NewFile("", src)
	ins, err := koi.GenerateIR(file)
	if err != nil {
		t.Fatal(err)
	}

	return ins.Instructions
}

func irCompare(ins []ir.Instruction, s string) bool {
	lines := strings.Split(strings.TrimSpace(ir.IrFmt(ins)), "\n")
	slines := strings.Split(strings.TrimSpace(s), "\n")

	for i, ins := range lines {
		if a, b := strings.TrimSpace(ins), strings.TrimSpace(slines[i]); a != b {
			fmt.Println(a, b)
			return false
		}
	}

	return true
}

func TestMain(t *testing.T) {
	ins := irFrom(t, `
		pub func main() int {
			return 42
		}
	`)

	expect := `
		PUB FUNC main -> int
			$0 i64 = 42
			RET $0
	`

	if !irCompare(ins, expect) {
		t.Errorf("expected equal")
	}
}

func TestMoreReturns(t *testing.T) {
	inputs := []string{
		`
			func foo() string {
				return "hello"
			}
		`,
		`
			func foo() float {
				return 1.0
			}
		`,
		`
			func foo() bool {
				return true
			}
		`,
	}

	expects := []string{
		`
			FUNC foo -> string
				$0 string = hello
				RET $0
		`,
		`
			FUNC foo -> float
				$0 f64 = 1.0
				RET $0
		`,
		`
			FUNC foo -> bool
				$0 bool = true
				RET $0
		`,
	}

	for i, in := range inputs {
		ins := irFrom(t, in)
		if !irCompare(ins, expects[i]) {
			t.Errorf("expected equal, case %d", i+1)
		}
	}
}
