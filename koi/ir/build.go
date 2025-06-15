package ir

import (
	"log"
	"strconv"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
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

func (b *Builder) Build() ([]Instruction, error) {
	b.tree.Walk(b)
	return b.ir, b.eh.Error()
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
	if node.T.Type != token.INT_LIT {
		log.Fatal("non-int types not implemented yet")
	}

	n, err := strconv.Atoi(node.T.Lexeme)
	if err != nil {
		panic("invalid integer literal")
	}

	b.ir = append(b.ir, Instruction{
		Op: STORE_INT64,
		Dest: Value{
			ID: b.curIdx,
		},
		Value: Value{
			Type:    Literal,
			Integer: n,
		},
	})
}

func (b *Builder) VisitIdent(node *ast.Ident) {

}
