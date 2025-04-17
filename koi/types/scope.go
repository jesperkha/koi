package types

type scope struct {
	cur    int
	scopes []map[string]Type
}

// Key for return type in scope, set to a string that can never show up as a token.
const returnType = "??RETURN"

func newScope() *scope {
	return &scope{
		cur:    0,
		scopes: make([]map[string]Type, 1),
	}
}

func (s *scope) push() {
	s.scopes = append(s.scopes, make(map[string]Type))
	s.cur++
}

func (s *scope) pop() {
	s.scopes = s.scopes[:s.cur]
	s.cur--
}

func (s *scope) get(name string) (Type, bool) {
	t, ok := s.scopes[s.cur][name]
	return t, ok
}

func (s *scope) set(name string, t Type) {
	s.scopes[s.cur][name] = t
}

func (s *scope) setReturnType(t Type) {
	s.set(returnType, t)
}

func (s *scope) getReturnType() Type {
	if t, ok := s.get(returnType); ok {
		return t
	}
	return &Primitive{Type: VOID}
}
