package main

import (
	"fmt"

	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/types"
)

func main() {
	a, err := koi.ParseFile("main.koi", nil)
	if err != nil {
		fmt.Println(err)
		return
	}

	c := types.NewChecker()
	c.Run(a)
}
