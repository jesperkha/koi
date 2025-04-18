package types

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
)

type (
	TypedAst struct {
		Nodes []Decl
	}

	Decl interface {
		ast.Node
	}

	Stmt interface {
		ast.Node
	}

	Expr interface {
		ast.Node
		Type() Type
	}
)

type (
	FuncDecl struct {
		Decl
		Public  bool
		Name    string
		RetType Type
		Params  []*Field
		Block   *Block
	}

	Field struct {
		Name string
		Type Type
	}
)

type (
	Block struct {
		Stmt
	}

	Return struct {
		Stmt
		E Expr
	}
)

type (
	Literal struct {
		Expr
		LitType   Type
		TokenType token.TokenType
		Value     string
	}
)

func (l *Literal) Type() Type { return l.LitType }
