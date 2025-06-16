package targets

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"

	"github.com/jesperkha/koi/koi/ir"
	"github.com/jesperkha/koi/koi/types"
)

type x86_64_builder struct {
	ins        []ir.Instruction
	r          types.TableReader
	buf        string
	header     string
	lineIndent int
}

func Build_x86_64(ir *ir.IR) {
	b := x86_64_builder{
		ins: ir.Instructions,
		r:   ir.Table,
	}

	b.writehdr("global _start")

	b.writeln(`
_start:
	call main
	mov r12, rax

	mov rax, 60
	mov rdi, r12
	syscall
	`)

	output := b.build()

	f, err := os.Create("bin/main.asm")
	if err != nil {
		log.Fatal(err)
	}
	f.Write([]byte(output))
	f.Close()

	cmd := exec.Command("nasm", "bin/main.asm", "-o", "bin/main.o", "-felf64")
	if _, err = cmd.Output(); err != nil {
		log.Fatal(err)
	}

	cmd = exec.Command("ld", "bin/main.o", "-o", "bin/main")
	if _, err = cmd.Output(); err != nil {
		log.Fatal(err)
	}
}

func (x *x86_64_builder) build() string {
	for _, sym := range x.r.Exported() {
		x.writehdr("global %s", sym.Name)
	}

	for _, ins := range x.ins {
		switch ins.Op {
		case ir.FUNC:
			x.writeln("%s:", ins.Name)
			x.indent()

		case ir.RET:
			x.writeln("ret")
			x.unindent()

		case ir.STORE_INT64:
			x.writeln("mov rax, %d", ins.Value.Value)
		}
	}

	return fmt.Sprintf("%s\n%s", x.header, x.buf)
}

func (x *x86_64_builder) writeln(s string, args ...any) {
	x.buf += fmt.Sprintf(strings.Repeat("	", x.lineIndent)+s+"\n", args...)
}

func (x *x86_64_builder) writehdr(s string, args ...any) {
	x.header += fmt.Sprintf(strings.Repeat("	", x.lineIndent)+s+"\n", args...)
}

func (x *x86_64_builder) indent() {
	x.lineIndent++
}

func (x *x86_64_builder) unindent() {
	x.lineIndent--
}
