package parser

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
)

func (p *Parser) parseExpr() ast.Expr {
	switch p.cur().Type {
	case token.NEWLINE:
		return nil
	case token.RBRACE:
		return nil
	default:
		return p.parseEquality()
	}
}

func (p *Parser) parseEquality() ast.Expr {
	return p.parseComparison()
}

func (p *Parser) parseComparison() ast.Expr {
	return p.parseTerm()
}

func (p *Parser) parseTerm() ast.Expr {
	return p.parseFactor()
}

func (p *Parser) parseFactor() ast.Expr {
	return p.parseUnary()
}

func (p *Parser) parseUnary() ast.Expr {
	return p.parseCall()
}

func (p *Parser) parseCall() ast.Expr {
	// Callee must be higher precedence, anything higher will create infinite
	// recursion. Chained calls are handled below.
	callee := p.parseGroup()

	// Chained calls are handled automatically by wrapping the previous
	// callee whenever we find another lparen.
	for p.match(token.LPAREN) {
		lparen := p.consume()
		args := []ast.Expr{}

		// Return early if no args
		if p.match(token.RPAREN) {
			goto finish_callee
		}

		for {
			expr := p.parseExpr()
			args = append(args, expr)
			if !p.match(token.COMMA) {
				break
			}

			p.next() // Comma
		}

		if !p.match(token.RPAREN) {
			p.err("expected ) after argument list")
		}

	finish_callee:
		rparen := p.consume()
		callee = &ast.Call{
			Callee: callee,
			LParen: lparen,
			Args:   args,
			RParen: rparen,
		}
	}

	return callee
}

func (p *Parser) parseGroup() ast.Expr {
	return p.parseLiteral()
}

func (p *Parser) parseLiteral() ast.Expr {
	if p.match(token.IDENT) {
		t := p.consume()
		return &ast.Ident{
			Name: t.Lexeme,
			T:    t,
		}
	}

	if !p.matchMany(token.INT_LIT, token.FLOAT_LIT, token.STRING_LIT, token.TRUE, token.FALSE, token.NIL, token.BYTE_LIT) {
		from := p.cur()
		p.gotoNewline()
		p.errFromTo(from, p.prev(), "invalid expression")
		return nil
	}

	t := p.consume()
	return &ast.Literal{
		T:     t,
		Value: t.Lexeme,
	}
}
