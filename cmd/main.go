package main

import (
	"log"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/ir"
)

func main() {
	a, tbl, err := koi.ParseFile("main.koi", nil)
	if err != nil {
		log.Fatal(err)
	}

	b := ir.NewBuilder(a, tbl)
	if err := b.Build(); err != nil {
		log.Fatal(err)
	}
}
