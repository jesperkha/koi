package ast

type Visitor interface {
	VisitFunc(node *Func)
	VisitBlock(node *Block)
	VisitReturn(node *Return)
	VisitExprStmt(node *ExprStmt)

	VisitCall(node *Call)
	VisitIdent(node *Ident)
	VisitLiteral(node *Literal)

	VisitPrimitiveType(node *PrimitiveType)
}

func (n *Func) Accept(v Visitor)          { v.VisitFunc(n) }
func (n *Block) Accept(v Visitor)         { v.VisitBlock(n) }
func (n *Literal) Accept(v Visitor)       { v.VisitLiteral(n) }
func (n *Return) Accept(v Visitor)        { v.VisitReturn(n) }
func (n *Ident) Accept(v Visitor)         { v.VisitIdent(n) }
func (n *Call) Accept(v Visitor)          { v.VisitCall(n) }
func (n *ExprStmt) Accept(v Visitor)      { v.VisitExprStmt(n) }
func (n *PrimitiveType) Accept(v Visitor) { v.VisitPrimitiveType(n) }
