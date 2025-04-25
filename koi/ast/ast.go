package ast

import "github.com/jesperkha/koi/koi/token"

type (
	Ast struct {
		// Declarations are the only top level statements in koi. They contain
		// all other statements and expressions. Eg. a function has a block
		// statement, which consists of multiple ifs and calls.
		Nodes []Decl
	}

	Node interface {
		Pos() token.Pos // Position of first token in node segment
		End() token.Pos // Position of last token in node segment

		// Accept a visitor to inspect this node. Must call the appropriate
		// visit method on the visitor for this node.
		Accept(v Visitor)
	}

	Expr interface {
		Node
	}

	Stmt interface {
		Node
	}

	// Declarations are not considered statements for linting purposes.
	// Functions, structs, enums etc are all top level statements, and
	// therefore declarations. This does not include variable declarations,
	// but does include constant declarations.
	Decl interface {
		Node
	}
)

func (t *Ast) Walk(v Visitor) {
	for _, decl := range t.Nodes {
		decl.Accept(v)
	}
}

type (
	// Single token identifier literal.
	Ident struct {
		Expr
		T token.Token
	}

	// Primitive literal, eg. string, number, bool etc.
	Literal struct {
		Expr
		T     token.Token
		Value string // Copied from the tokens Lexeme value for ease of use
	}
)

type (
	Return struct {
		Stmt
		Ret token.Token
		E   Expr // Is nil when no return value is specified
	}

	Block struct {
		Stmt
		Empty  bool // If the Stmts list is empty
		LBrace token.Token
		Stmts  []Stmt
		RBrace token.Token
	}
)

type (
	Func struct {
		Decl
		Public  bool
		Name    token.Token
		Params  *NamedTuple
		RetType Type
		Block   *Block
	}
)

// Other AST node types.
// These types are not strictly part of the language spec and are not valid
// expressions or statements by themselves, but serve as containers for
// common features in other nodes.
type (
	// A field is a name-type combination. Eg. "foo int"
	Field struct {
		Node
		Name token.Token
		Type Type
	}

	// A named tuple is a list of fields within parenthesis.
	// Eg. "(name string, age int)"
	NamedTuple struct {
		Empty  bool // If the fields list is empty
		LParen token.Token
		Fields []*Field
		RParen token.Token
	}
)

func (i *Ident) Pos() token.Pos { return i.T.Pos }
func (i *Ident) End() token.Pos { return i.T.Pos }

func (l *Literal) Pos() token.Pos { return l.T.Pos }
func (l *Literal) End() token.Pos { return l.T.EndPos }

func (r *Return) Pos() token.Pos { return r.Ret.Pos }
func (r *Return) End() token.Pos {
	if r.E != nil {
		return r.E.End()
	}
	return r.Ret.EndPos
}

func (b *Block) Pos() token.Pos { return b.LBrace.Pos }
func (b *Block) End() token.Pos { return b.LBrace.EndPos }

func (f *Func) Pos() token.Pos { return f.Name.Pos }
func (f *Func) End() token.Pos { return f.Name.EndPos }

func (f *Field) Pos() token.Pos { return f.Name.Pos }
func (f *Field) End() token.Pos { return f.Type.End() }
