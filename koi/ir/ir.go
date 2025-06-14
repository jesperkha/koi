package ir

import "github.com/jesperkha/koi/koi/types"

type OpCode int

const (
	NOP OpCode = iota

	FUNC
	RET
	CONST
	PUB
)

type Instruction struct {
	Op OpCode

	Name    string
	Public  bool
	RetType types.Type

	Dest  Value
	Value Value
}

const (
	Immediate = iota
	Constant
	Variable
)

type Value struct {
	Type int
	Idx  int

	Integer int
}
