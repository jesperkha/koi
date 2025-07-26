package parser

import (
	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
)

func (p *Parser) parseDecl() ast.Decl {
	switch p.cur().Type {
	case token.EOF:
		return nil

	case token.NEWLINE:
		p.next()
		return p.parseDecl()

	case token.FUNC, token.PUB:
		return p.parseFunc()
	}

	// Unrecoverable error
	p.err("unknown top level statement")
	return nil
}

func (p *Parser) parseFunc() *ast.Func {
	public := false
	if p.match(token.PUB) {
		public = true
		p.next()
	}

	p.next() // Func keyword which is guaranteed

	name := p.expect(token.IDENT)
	params := p.parseNamedTuple()

	if p.match(token.LBRACE) {
		p.err("expected return type")
	}

	// TODO: parse typeTuple when multi-return is added
	typ := p.parseType()
	block := p.parseBlock()

	return &ast.Func{
		Public:  public,
		Name:    name,
		Params:  params,
		RetType: typ,
		Block:   block,
	}
}

func (p *Parser) parseNamedTuple() *ast.NamedTuple {
	if p.panicMode {
		return nil
	}

	lparen := p.expect(token.LPAREN)

	if p.match(token.RPAREN) {
		rparen := p.consume()
		return &ast.NamedTuple{
			Empty:  true,
			LParen: lparen,
			RParen: rparen,
		}
	}

	tuple := &ast.NamedTuple{}

	for !p.eofOrPanic() {
		name := p.expect(token.IDENT)
		typ := p.parseType()

		tuple.Fields = append(tuple.Fields, &ast.Field{
			Name: name,
			Type: typ,
		})

		if p.match(token.RPAREN) {
			break
		}

		p.expect(token.COMMA)
	}

	p.next() // Right paren
	return tuple
}
