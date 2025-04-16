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

// Expression types
type (
	// Invalid expression. Simply a range of tokens containing a syntax error.
	BadExpr struct {
		Expr
		From, To token.Token
	}

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

// Statement types
type (
	Return struct {
		Stmt
		Ret token.Token
		E   Expr
	}

	Block struct {
		Stmt
		Empty  bool // If the Stmts list is empty
		LBrace token.Token
		Stmts  []Stmt
		RBrace token.Token
	}
)

// Declaration types
type (
	// Function declaration.
	Func struct {
		Decl
		Public bool
		Name   token.Token
		Params *NamedTuple
		Type   *Type
		Block  *Block
	}
)

// Other AST node types.
// These types are not strictly part of the language spec and are not valid
// expressions or statements by themselves, but serve as containers for
// common features in other nodes.
type (
	// A primitive or compound type
	Type struct {
		// The primitive or user defined part of the type.
		// Eg. []string -> string, []Person -> Person, int -> int
		T token.Token
	}

	// A field is a name-type combination. Eg. "foo int"
	Field struct {
		Name token.Token
		Type *Type
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

func (b *BadExpr) Pos() token.Pos { return b.From.Pos }
func (b *BadExpr) End() token.Pos { return b.To.Pos }

func (i *Ident) Pos() token.Pos { return i.T.Pos }
func (i *Ident) End() token.Pos { return i.T.Pos }

func (l *Literal) Pos() token.Pos { return l.T.Pos }
func (l *Literal) End() token.Pos { return l.T.Pos }

func (r *Return) Pos() token.Pos { return r.E.Pos() }
func (r *Return) End() token.Pos { return r.E.End() }
