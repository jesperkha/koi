package ir

type OpCode int

type Intruction struct {
	Op int
}

const (
	NOP OpCode = iota
)
