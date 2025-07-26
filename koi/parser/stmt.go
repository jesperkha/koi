package parser

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
)

func (p *Parser) parseStmt() ast.Stmt {
	if p.panicMode {
		return nil
	}

	switch p.cur().Type {
	case token.RETURN:
		return p.parseReturn()
	case token.LBRACE:
		return p.parseBlock()

	default:
		return &ast.ExprStmt{
			E: p.parseExpr(),
		}
	}
}

func (p *Parser) parseType() ast.Type {
	if p.panicMode {
		return nil
	}

	start := p.cur() // For error tracking

	if p.match(token.NEWLINE) {
		p.err("expected type")
		return nil
	}

	// Primitive type with only identifier.
	if p.matchMany(token.STRING, token.VOID, token.INT, token.FLOAT, token.BYTE, token.BOOL) {
		t := p.consume()
		return &ast.PrimitiveType{
			T:    t,
			Kind: ast.TokenToTypeKind(t.Type),
		}
	}

	// Array type.
	// if p.match(token.LBRACK) {
	// 	lbrack := p.consume()
	// 	if !p.match(token.RBRACK) {
	// 		p.err("expected ] to complete array type")
	// 		return nil
	// 	}

	// 	p.next() // ]
	// 	typ := p.parseType()

	// 	return &ast.ArrayType{
	// 		LBrack: lbrack.Pos,
	// 		Type:   typ,
	// 	}
	// }

	p.errFromTo(start, p.cur(), "invalid type")
	return nil
}

func (p *Parser) parseReturn() *ast.Return {
	ret := p.consume() // Return keyword is guaranteed

	if p.match(token.NEWLINE) {
		p.next()
		return &ast.Return{
			Ret: ret,
			E:   nil,
		}
	}

	expr := p.parseExpr()
	return &ast.Return{
		E:   expr,
		Ret: ret,
	}
}

func (p *Parser) parseBlock() *ast.Block {
	if p.panicMode {
		return nil
	}

	lbrace := p.expect(token.LBRACE)
	stmts := []ast.Stmt{}

	for !p.eofOrPanic() && !p.match(token.RBRACE) {
		if p.match(token.NEWLINE) {
			p.next()
			continue
		}

		s := p.parseStmt()
		if !p.matchMany(token.NEWLINE, token.RBRACE) {
			from := p.cur()
			p.gotoNewline()
			p.errFromTo(from, p.prev(), "expected end of statement")
			continue
		}

		stmts = append(stmts, s)
	}

	rbrace := p.expect(token.RBRACE)
	return &ast.Block{
		LBrace: lbrace,
		Stmts:  stmts,
		RBrace: rbrace,
		Empty:  len(stmts) == 0,
	}
}
