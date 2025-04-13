package parser

import (
	"github.com/jesperkha/koi/koi"
	"github.com/jesperkha/koi/koi/token"
)

type Parser struct {
	handler *koi.ErrorHandler
	file    *token.File
	toks    []token.Token
	pos     int // Current token being looked at
	base    int // Token at start of current statement
}

func New(file *token.File, toks []token.Token, errHandler *koi.ErrorHandler) *Parser {
	return &Parser{
		handler: errHandler,
		toks:    toks,
		file:    file,
	}
}
