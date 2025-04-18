package types

type Context struct {
	idx    int
	scopes []Scope
}

type Scope struct {
	maps map[string]Type // Map of identifier name to type

	// The current scopes expected return type, used for function bodies.
	// Defaults to void and is never nil. Child scopes will inherit this value.
	ret Type

	// True if there has been a return statement in the current scope (not
	// counting child scopes such as if-statements or other blocks).
	//
	// This value being true makes all succeeding statements in the current
	// scope unreachable. This value is set by calling MarkReturned().
	hasReturned bool
}

func NewContext() *Context {
	ctx := &Context{
		idx:    0,
		scopes: []Scope{},
	}

	ctx.Push()  // Push base scope
	ctx.idx = 0 // Reset after first push
	return ctx
}

func (c *Context) Push() {
	// Inherit parent return type. Base scope is set to void.
	var ret Type
	if c.idx == 0 {
		ret = &Primitive{Type: VOID}
	} else {
		ret = c.GetReturnType()
	}

	scope := Scope{
		ret:  ret,
		maps: make(map[string]Type),
	}

	c.scopes = append(c.scopes, scope)
	c.idx++
}

func (c *Context) Pop() {
	c.scopes = c.scopes[:c.idx]
	c.idx--
}

// Set maps the given name to the type t for the current scope. Overrides any
// existing value.
func (c *Context) Set(name string, t Type) {
	c.cur().maps[name] = t
}

// Get the type mapped to name. Returns ok bool if found.
func (c *Context) Get(name string) (Type, bool) {
	t, ok := c.cur().maps[name]
	return t, ok
}

// Mark the current scope as having returned, making all succeeding statements
// unreachable. Does nothing if value is already set.
func (c *Context) MarkReturned() {
	c.cur().hasReturned = true
}

// HasReturned reports whether the current scope has returned yet.
func (c *Context) HasReturned() bool {
	return c.cur().hasReturned
}

// Set the current scopes return type.
func (c *Context) SetReturnType(t Type) {
	c.cur().ret = t
}

// Get the current scopes return type. Defaults to void and is never nil.
func (c *Context) GetReturnType() Type {
	return c.cur().ret
}

func (c *Context) cur() *Scope {
	return &c.scopes[c.idx]
}
