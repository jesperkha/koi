package main

import (
	"fmt"
	"log"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ast"
)

func main() {
	a, err := koi.ParseFile("main.koi", nil)
	if err != nil {
		log.Fatal(err)
	}

	v := ast.NewDebugVisitor()
	a.Walk(v)
	fmt.Println(v.String())
}
