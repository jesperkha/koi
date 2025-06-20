package ir

import (
	"github.com/jesperkha/koi/koi/types"
)

// The IR type represents the intermediate representation for a single file.
type IR struct {
	Instructions []Instruction
	Table        types.TableReader
}

type OpCode int

const (
	NOP OpCode = iota

	FUNC
	RET

	STORE_INT64
	STORE_STR
	STORE_FLOAT64
	STORE_BOOL
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
	Literal = iota
	Variable
)

type Value struct {
	ID    int
	Type  int
	Value string
}
