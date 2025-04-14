package main

import (
	"fmt"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ast"
)

func main() {
	a, err := koi.ParseFile("", "func main(a int, b float) string {} func foo() {}")
	if err != nil {
		fmt.Println(err)
	}

	ast.Print(a)
}
