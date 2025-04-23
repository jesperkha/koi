package ast

type Visitor interface {
	VisitFunc(node *Func)
	VisitBlock(node *Block)
	VisitLiteral(node *Literal)
	VisitType(node Type)
	VisitReturn(node *Return)
	VisitPrimitive(node *PrimitiveType)
}

func (n *Func) Accept(v Visitor)          { v.VisitFunc(n) }
func (n *Block) Accept(v Visitor)         { v.VisitBlock(n) }
func (n *Literal) Accept(v Visitor)       { v.VisitLiteral(n) }
func (n *Return) Accept(v Visitor)        { v.VisitReturn(n) }
func (n *PrimitiveType) Accept(v Visitor) { v.VisitPrimitive(n) }
