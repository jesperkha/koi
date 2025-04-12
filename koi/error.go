package koi

type ErrorHandler struct {
	errs []error
}

// Add an error to the handler. It is up to the handler to either report
// it right away, wait for later, or ignore the error completely.
func (e *ErrorHandler) Add(err error) {
	e.errs = append(e.errs, err)
}

// Errors returns a list of all accumulated errors.
func (e *ErrorHandler) Errors() []error {
	return e.errs
}
