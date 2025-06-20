package ast

type Visitor interface {
	VisitFunc(node *Func)
	VisitBlock(node *Block)
	VisitLiteral(node *Literal)
	VisitReturn(node *Return)
	VisitIdent(node *Ident)
	VisitCall(node *Call)
	VisitExprStmt(node *ExprStmt)
}

func (n *Func) Accept(v Visitor)     { v.VisitFunc(n) }
func (n *Block) Accept(v Visitor)    { v.VisitBlock(n) }
func (n *Literal) Accept(v Visitor)  { v.VisitLiteral(n) }
func (n *Return) Accept(v Visitor)   { v.VisitReturn(n) }
func (n *Ident) Accept(v Visitor)    { v.VisitIdent(n) }
func (n *Call) Accept(v Visitor)     { v.VisitCall(n) }
func (n *ExprStmt) Accept(v Visitor) { v.VisitExprStmt(n) }
