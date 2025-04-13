package ast

import "github.com/jesperkha/koi/koi/token"

type (
	Ast struct {
	}

	Node interface {
		Pos() token.Pos // Position of first token in node segment
		End() token.Pos // Position of last token in node segment
	}

	// An expression is any sequence of tokens that can be evaluated into a single value.
	Expr interface {
		Node
	}

	// A statement is any control flow, logical, or variable statement.
	// Eg. if, else, return etc. Declarations are not considered statements
	// for linting purposes.
	Stmt interface {
		Node
	}

	// A declaration is a top level function or type declaration (not variable).
	Decl interface {
		Node
	}

	// Expression types

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

	// Statement types

	Return struct {
		Stmt
		E Expr
	}

	// Declaration types

	// Function declaration.
	Func struct {
		Decl
		Public bool
		Name   token.Token
		Params NamedTuple
	}

	// Other AST node types.
	// These types are not strictly part of the language spec and are not valid
	// expressions or statements by themselves, but serve as containers for
	// common features in other nodes.

	// A primitive or compound type
	Type struct {
		// The primitive or user defined part of the type.
		// Eg. []string -> string, []Person -> Person, int -> int
		T token.Token
	}

	// A field is a name-type combination. Eg. "foo int"
	Field struct {
		Name token.Token
		Type Type
	}

	// A named tuple is a list of fields within parenthesis.
	// Eg. "(name string, age int)"
	NamedTuple struct {
		LParen token.Token
		Fields []Field
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
