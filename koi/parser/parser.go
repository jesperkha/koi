package parser

import (
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

	return nil
}

func (p *Parser) Error() error {
	return p.errors.Error()
}
