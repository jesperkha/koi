package util

import (
	"errors"
)

type ErrorList struct {
	errs []error
}

func (e *ErrorList) Add(err error) {
	e.errs = append(e.errs, err)
}

func (e *ErrorList) Errors() []error {
	return e.errs
}

func (e *ErrorList) Error() error {
	return errors.Join(e.errs...)
}
