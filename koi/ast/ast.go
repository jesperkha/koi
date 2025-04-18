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

	Type interface {
		Node
		// Get string representation of type, identical to the type syntax.
		String() string
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

func (b *BadExpr) Pos() token.Pos { return b.From.Pos }
func (b *BadExpr) End() token.Pos { return b.To.Pos }

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

type TypeKind int

const (
	VOID TypeKind = iota
	STRING
	BYTE
	INT32
	FLOAT32
	ARRAY
)

type (
	PrimitiveType struct {
		T token.Token
	}

	ArrayType struct {
		LBrack token.Pos
		Type   Type
	}
)

func (p *PrimitiveType) String() string { return p.T.Lexeme }
func (p *PrimitiveType) Pos() token.Pos { return p.T.Pos }
func (p *PrimitiveType) End() token.Pos { return p.T.EndPos }

func (a *ArrayType) String() string { return "[]" + a.Type.String() }
func (a *ArrayType) Pos() token.Pos { return a.LBrack }
func (a *ArrayType) End() token.Pos { return a.Type.End() }
