package parser

import (
	"fmt"
	"log"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/util"
)

type Parser struct {
	errors util.ErrorList
	file   *token.File
	toks   []token.Token
	pos    int // Current token being looked at
	base   int // Token at start of current statement
}

func New(file *token.File, toks []token.Token) *Parser {
	return &Parser{
		toks:   toks,
		file:   file,
		errors: util.ErrorList{},
	}
}

func (p *Parser) Parse() *ast.Ast {
	if p.eof() {
		return &ast.Ast{}
	}

	// True if we found a pub keyword. Reset every iteration.
	public := false
	nodes := []ast.Decl{}

loop:
	for !p.eof() {
		var n ast.Decl

		switch p.cur().Type {
		case token.EOF:
			break loop

		case token.PUB:
			public = true
			p.next()
			continue

		case token.FUNC:
			n = p.parseFunc(public)

		default:
			// Unrecoverable error
			p.err("unknown top level statement, found '%s'", p.cur().Lexeme)
			break loop
		}

		nodes = append(nodes, n) // n is never nil
		public = false
	}

	return &ast.Ast{
		Nodes: nodes,
	}
}

func (p *Parser) Error() error {
	return p.errors.Error()
}

func (p *Parser) cur() token.Token {
	if p.eof() {
		return token.Token{
			Type: token.EOF,
			Eof:  true,
		}
	}

	return p.toks[p.pos]
}

func (p *Parser) next() {
	p.pos++
}

func (p *Parser) eof() bool {
	return p.pos >= len(p.toks)
}

// Peek next token. Returns EOF token if at end of token list.
func (p *Parser) peek() token.Token {
	if p.pos+1 >= len(p.toks) {
		return token.Token{
			Type: token.EOF,
			Eof:  true,
		}
	}

	return p.toks[p.pos+1]
}

// Same as next but also returns the token it consumed.
func (p *Parser) consume() token.Token {
	t := p.cur()
	p.next()
	return t
}

func (p *Parser) err(f string, args ...any) {
	// TODO: pretty error messages
	p.errors.Add(fmt.Errorf(f, args...))
}

func (p *Parser) expect(typ token.TokenType) token.Token {
	t := p.consume()

	if t.Type != typ {
		p.err("expected %s", token.String(typ))
	}

	return t
}

func (p *Parser) parseFunc(public bool) *ast.Func {
	// We know that the first token is FUNC
	p.next()

	name := p.expect(token.IDENT)
	params := p.parseNamedTuple()
	retType := &ast.Type{}

	if p.cur().Type != token.LBRACE {
		retType = p.parseType()
	}

	if p.cur().Type != token.LBRACE {
		p.err("expected block after function declaration")
	}

	block := p.parseBlock()

	return &ast.Func{
		Public: public,
		Name:   name,
		Params: params,
		Type:   retType,
		Block:  block,
	}
}

func (p *Parser) parseNamedTuple() *ast.NamedTuple {
	lparen := p.expect(token.LPAREN)

	if p.cur().Type == token.RPAREN {
		rparen := p.consume()
		return &ast.NamedTuple{
			Empty:  true,
			LParen: lparen,
			RParen: rparen,
		}
	}

	log.Fatal("parseNamedTuple not implemented for actual tuple values")
	return nil
}

func (p *Parser) parseType() *ast.Type {
	log.Fatal("parseType not implemented")
	return nil
}

func (p *Parser) parseBlock() *ast.Block {
	lbrace := p.expect(token.LBRACE)

	if p.cur().Type == token.RBRACE {
		rbrace := p.consume()
		return &ast.Block{
			Empty:  true,
			LBrace: lbrace,
			RBrace: rbrace,
		}
	}

	log.Fatal("parseBlock with actual statements not implemented")
	return nil
}
