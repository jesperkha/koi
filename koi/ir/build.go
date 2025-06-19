package ir

import (
	"fmt"
	"log"
	"strings"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/types"
	"github.com/jesperkha/koi/koi/util"
)

// Builder implements the Visitor interface
type Builder struct {
	eh     *util.ErrorHandler
	tree   *ast.Ast
	tr     types.TableReader
	ir     []Instruction
	ctr    int
	curIdx int // Current index to assign result to
}

func NewBuilder(tree *ast.Ast, reader types.TableReader) *Builder {
	return &Builder{
		eh:   util.NewErrorHandler(),
		tree: tree,
		tr:   reader,
	}
}

// Get next available index
func (b *Builder) idx() int {
	prev := b.ctr
	b.ctr++
	return prev
}

func (b *Builder) setIdx(idx int) {
	b.curIdx = idx
}

func (b *Builder) Build() (ir *IR, err error) {
	b.tree.Walk(b)
	return &IR{
		Instructions: b.ir,
		Table:        b.tr,
	}, b.eh.Error()
}

func (b *Builder) VisitFunc(node *ast.Func) {
	funcName := node.Name.Lexeme

	b.ir = append(b.ir, Instruction{
		Op:      FUNC,
		Name:    funcName,
		Public:  node.Public,
		RetType: b.tr.Get(funcName).Type,
	})

	for range node.Params.Fields {
		log.Fatal("param ir not implemented")
	}

	node.Block.Accept(b)
}

func (b *Builder) VisitBlock(node *ast.Block) {
	b.tr.Push(node)
	for _, stmt := range node.Stmts {
		stmt.Accept(b)
	}
	b.tr.Pop()
}

func (b *Builder) VisitReturn(node *ast.Return) {
	if node.E != nil {
		result := b.idx()
		b.setIdx(result)
		node.E.Accept(b)

		b.ir = append(b.ir, Instruction{
			Op: RET,
			Value: Value{
				Type: Variable,
				ID:   result,
			},
		})
	} else {
		log.Fatal("noval return not implemented")
	}
}

func (b *Builder) VisitLiteral(node *ast.Literal) {
	kind := ast.TokenToTypeKind(node.T.Type)
	op := NOP
	value := node.Value

	switch kind {
	case ast.INT:
		op = STORE_INT64
	case ast.FLOAT:
		op = STORE_FLOAT64
	case ast.STRING:
		op = STORE_STR
		value = strings.Trim(node.Value, "\"")
	case ast.BOOL:
		op = STORE_BOOL

	default:
		panic("unsupported literal kind: " + fmt.Sprintf("kind=%d", kind))
	}

	b.ir = append(b.ir, Instruction{
		Op:   op,
		Dest: Value{ID: b.curIdx},
		Value: Value{
			Type:  Literal,
			Value: value,
		},
	})

}

func (b *Builder) VisitIdent(node *ast.Ident) {

}

func (b *Builder) VisitCall(node *ast.Call) {

}
