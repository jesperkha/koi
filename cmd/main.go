package main

import (
	"github.com/jesperkha/koi/koi/scanner"
	"github.com/jesperkha/koi/koi/token"
)

func main() {
	// a, err := koi.ParseFile("main.koi", nil)
	// if err != nil {
	// 	fmt.Println(err)
	// } else {
	// 	ast.Print(a)
	// }

	s := scanner.New(&token.File{}, []byte("// hello\nworld"))
	token.Print(s.ScanAll())

}
