package main

import (
	"log"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
)

func main() {
	file := token.NewFile("main.koi", nil)
	tree, _, err := koi.Parse(file)
	if err != nil {
		log.Fatal(err)
	}

	ast.Print(tree)

	// ir, err := koi.GenerateIR(file)
	// if err != nil {
	// 	log.Fatal(err)
	// }

	// targets.Build_x86_64(ir)
}
