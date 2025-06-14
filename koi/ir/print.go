package ir

import "fmt"

func PrintIR(ir []Instruction) {
	for _, op := range ir {
		switch op.Op {
		case FUNC:
			if op.Public {
				fmt.Printf("PUB FUNC %s -> %s\n", op.Name, op.RetType.String())
			} else {
				fmt.Printf("FUNC %s -> %s\n", op.Name, op.RetType.String())
			}

		default:
			fmt.Println("unknown op")
		}
	}
}
