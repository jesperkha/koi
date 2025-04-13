package main

import (
	"fmt"

	"github.com/jesperkha/koi/koi"
)

func main() {
	_, err := koi.ParseFile("", "func main(a int) {}")
	if err != nil {
		fmt.Println(err)
	}
}
