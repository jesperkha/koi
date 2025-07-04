package parser

import (
	"fmt"
	"slices"

	"github.com/jesperkha/koi/koi/ast"
	"github.com/jesperkha/koi/koi/token"
	"github.com/jesperkha/koi/koi/util"
)

type Parser struct {
	NumErrors int

	errors util.ErrorHandler
	file   *token.File
	toks   []token.Token
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

func New(file *token.File, toks []token.Token) *Parser {
	return &Parser{
		toks:   toks,
		file:   file,
		errors: util.ErrorHandler{},
	}
}

func (p *Parser) Parse() *ast.Ast {
	if p.eof() {
		return &ast.Ast{}
	}

	nodes := []ast.Decl{}

loop:
	for !p.eof() {
		var n ast.Decl

		switch p.cur().Type {
		case token.EOF:
			break loop

		case token.NEWLINE:
			p.next()
			continue

		case token.FUNC, token.PUB:
			n = p.parseFunc()

		default:
			// Unrecoverable error
			p.err("unknown top level statement")
			break loop
		}

		nodes = append(nodes, n) // n is never nil
	}

	return &ast.Ast{Nodes: nodes}
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

func (p *Parser) matchMany(types ...token.TokenType) bool {
	return slices.Contains(types, p.cur().Type)
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

	row := from.Pos.Row
	msg := fmt.Sprintf(format, args...)
	start := from.Pos.Col
	end := to.EndPos.Col
	p.errors.Pretty(row+1, p.file.Line(row), msg, start, end)

	p.panic()
	p.NumErrors++
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
	if !p.matchMany(types...) {
		p.err("expected %s", label)
		return p.cur()
	}

	return p.consume()
}

// Skips to next newline token and returns it. Does the same in case of eof.
func (p *Parser) gotoNewline() token.Token {
	for !p.eof() && !p.match(token.NEWLINE) {
		p.next()
	}
	return p.cur()
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

		// Return early if no args
		if p.match(token.RPAREN) {
			rparen := p.consume()
			return &ast.Call{
				Callee: callee,
				LParen: lparen,
				Args:   []ast.Expr{},
				RParen: rparen,
			}
		}

		args := []ast.Expr{}
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
