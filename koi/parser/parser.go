package parser

import (
	"fmt"
	"slices"
	"strings"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Parser struct {
	errors util.ErrorList
	file   *token.File
	toks   []token.Token
	src    []byte
	pos    int // Current token being looked at

	// Panic mode occurs when the parser encounters an unknown token sequence
	// and needs to synchronize to a 'clean' state. When panic mode starts,
	// the base position is set to the current position. When in panic mode
	// all err() calls are ignored.
	//
	// Functions which parse statements should have a check at the top for
	// panicMode, and return early with an invalid statement if set.
	panicMode bool
	base      int
}

func New(file *token.File, toks []token.Token, src []byte) *Parser {
	return &Parser{
		toks:   toks,
		file:   file,
		src:    src,
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
			p.err("unknown top level statement")
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

// Enter panic mode. Set base position. All errors are ignored until panic mode
// is recovered with recover().
func (p *Parser) panic() {
	p.panicMode = true
	p.base = p.pos
}

// Recover from panic mode. Sets pos to base and looks for next statement keyword.
func (p *Parser) recover() {
	p.panicMode = false
	p.pos = p.base

	for !p.eof() {
		switch p.cur().Type {
		case token.IF, token.FUNC, token.FOR, token.RETURN:
			return
		}

		p.next()
	}
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

// Shorthand for p.cur().Type == token.X
func (p *Parser) match(t token.TokenType) bool {
	return p.cur().Type == t
}

func (p *Parser) next() {
	p.pos++
}

func (p *Parser) eof() bool {
	return p.pos >= len(p.toks)
}

func (p *Parser) eofOrPanic() bool {
	return p.eof() || p.panicMode
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

func (p *Parser) prev() token.Token {
	if p.pos == 0 {
		return token.Token{
			Type: token.EOF,
			Eof:  true,
		}
	}
	return p.toks[p.pos-1]
}

// Same as next but also returns the token it consumed.
func (p *Parser) consume() token.Token {
	t := p.cur()
	p.next()
	return t
}

func (p *Parser) errFromTo(from token.Token, to token.Token, format string, args ...any) {
	if p.panicMode {
		return
	}

	lineStr := p.src[from.Pos.LineBegin : util.FindEndOfLine(p.src, from.Pos.LineBegin)+1]
	length := to.EndPos.Col - from.Pos.Col

	err := ""
	err += fmt.Sprintf("error: %s\n", fmt.Sprintf(format, args...))
	err += fmt.Sprintf("%3d | %s\n", from.Pos.Row+1, lineStr)
	err += fmt.Sprintf("    | %s%s\n", strings.Repeat(" ", from.Pos.Col), strings.Repeat("^", length))

	p.errors.Add(fmt.Errorf("%s", err))
	p.panic()
}

func (p *Parser) err(format string, args ...any) {
	p.errFromTo(p.cur(), p.cur(), format, args...)
}

// Expects current token to be typ. Only consumes it if correct, otherwise throws error.
func (p *Parser) expect(typ token.TokenType) token.Token {
	if !p.match(typ) {
		p.err("expected %s", token.String(typ))
		return p.cur()
	}

	return p.consume()
}

// Same as expect, but takes multiple types to compare. Label is what to call
// the expected tokens for errors.
func (p *Parser) expectMany(label string, types ...token.TokenType) token.Token {
	if !slices.Contains(types, p.cur().Type) {
		p.err("expected %s", label)
		return p.cur()
	}

	return p.consume()
}

func (p *Parser) parseFunc(public bool) *ast.Func {
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
		Public: public,
		Name:   name,
		Params: params,
		Type:   typ,
		Block:  block,
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

func (p *Parser) parseType() *ast.Type {
	if p.panicMode {
		return nil
	}

	t := p.expectMany("type", token.STRING_T, token.VOID, token.INT, token.FLOAT, token.BYTE)

	return &ast.Type{
		T: t,
	}
}

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
		p.err("invalid statement")
	}

	return nil
}

func (p *Parser) parseReturn() *ast.Return {
	ret := p.consume() // Return keyword is guaranteed

	if p.match(token.SEMI) {
		p.next()
		return &ast.Return{
			Ret: ret,
			E:   &ast.NoExpr{},
		}
	}

	expr := p.parseExpr()
	p.expect(token.SEMI)

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
	if p.match(token.RBRACE) {
		rbrace := p.consume()
		return &ast.Block{
			Empty:  true,
			LBrace: lbrace,
			RBrace: rbrace,
		}
	}

	stmts := []ast.Stmt{}
	for !p.eofOrPanic() && !p.match(token.RBRACE) {
		s := p.parseStmt()
		stmts = append(stmts, s)
	}

	rbrace := p.expect(token.RBRACE)
	return &ast.Block{
		LBrace: lbrace,
		Stmts:  stmts,
		RBrace: rbrace,
	}
}

func (p *Parser) parseExpr() ast.Expr {
	switch p.cur().Type {
	case token.NUMBER, token.STRING, token.TRUE, token.FALSE, token.NIL:
		t := p.consume()
		return &ast.Literal{
			T:     t,
			Value: t.Lexeme,
		}

	default:
		p.err("expected expression")
	}

	return nil
}
