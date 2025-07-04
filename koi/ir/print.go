package ir

import (
	"fmt"
	"strings"
)

func PrintIR(ir []Instruction) {
	fmt.Println(IrFmt(ir))
}

func IrFmt(ir []Instruction) string {
	s := ""
	indent := 0

	for _, op := range ir {
		s += strings.Repeat("  ", indent)
		switch op.Op {
		case FUNC:
			if op.Public {
				s += fmt.Sprintf("PUB FUNC %s -> %s\n", op.Name, op.RetType.String())
			} else {
				s += fmt.Sprintf("FUNC %s -> %s\n", op.Name, op.RetType.String())
			}
			indent++

		case STORE_INT64:
			if op.Value.Type == Literal {
				s += fmt.Sprintf("$%d i64 = %s\n", op.Dest.ID, op.Value.Value)
			} else {
				s += fmt.Sprintf("$%d i64 = $%d\n", op.Dest.ID, op.Value.ID)
			}

		case STORE_FLOAT64:
			if op.Value.Type == Literal {
				s += fmt.Sprintf("$%d f64 = %s\n", op.Dest.ID, op.Value.Value)
			} else {
				s += fmt.Sprintf("$%d f64 = $%d\n", op.Dest.ID, op.Value.ID)
			}

		case STORE_STR:
			if op.Value.Type == Literal {
				s += fmt.Sprintf("$%d string = %s\n", op.Dest.ID, op.Value.Value)
			} else {
				s += fmt.Sprintf("$%d string = $%d\n", op.Dest.ID, op.Value.ID)
			}

		case STORE_BOOL:
			if op.Value.Type == Literal {
				s += fmt.Sprintf("$%d bool = %s\n", op.Dest.ID, op.Value.Value)
			} else {
				s += fmt.Sprintf("$%d bool = $%d\n", op.Dest.ID, op.Value.ID)
			}

		case RET:
			s += fmt.Sprintf("RET $%d\n", op.Value.ID)

		default:
			s += fmt.Sprintf("unknown op")
		}
	}

	return s
}
