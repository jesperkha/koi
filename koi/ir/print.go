package ir

import (
	"fmt"
	"strings"
)

func PrintIR(ir []Instruction) {
	indent := 0
	for _, op := range ir {
		fmt.Print(strings.Repeat("  ", indent))
		switch op.Op {
		case FUNC:
			if op.Public {
				fmt.Printf("PUB FUNC %s -> %s\n", op.Name, op.RetType.String())
			} else {
				fmt.Printf("FUNC %s -> %s\n", op.Name, op.RetType.String())
			}
			indent++

		case CONST:
			if op.Value.Type == Immediate {
				fmt.Printf("$%d = %s\n", op.Value.Idx, fmt.Sprintf("%d", op.Value.Integer))
			}

		case RET:
			fmt.Printf("RET $%d\n", op.Value.Idx)

		default:
			fmt.Println("unknown op")
		}
	}
}
