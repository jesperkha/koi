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
