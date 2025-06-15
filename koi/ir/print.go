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

		case STORE_INT64:
			if op.Value.Type == Literal {
				fmt.Printf("$%d i64 = %s\n", op.Dest.ID, fmt.Sprintf("%d", op.Value.Integer))
			} else {
				fmt.Printf("$%d i64 = $%d\n", op.Dest.ID, op.Value.ID)
			}

		case RET:
			fmt.Printf("RET $%d\n", op.Value.ID)

		default:
			fmt.Println("unknown op")
		}
	}
}
