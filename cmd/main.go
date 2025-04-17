package main

import (
	"fmt"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ast"
)

func main() {
	a, err := koi.ParseFile("main.koi", nil)
	if err != nil {
		fmt.Println(err)
	} else {
		ast.Print(a)
	}
}
