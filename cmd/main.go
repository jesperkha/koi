package main

import (
	"log"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/compile/targets"
	"github.com/jesperkha/koi/koi/ir"
)

func main() {
	a, tbl, err := koi.ParseFile("main.koi", nil)
	if err != nil {
		log.Fatal(err)
	}

	b := ir.NewBuilder(a, tbl)
	ops, err := b.Build()
	if err != nil {
		log.Fatal(err)
	}

	targets.Build_x86_64(tbl, ops)
}
