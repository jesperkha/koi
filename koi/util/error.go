package util

import (
	"errors"
	"fmt"
	"strings"
)

type ErrorHandler struct {
	errs []error
}

func NewErrorHandler() *ErrorHandler {
	return &ErrorHandler{}
}

func (e *ErrorHandler) Add(err error) {
	e.errs = append(e.errs, err)
}

func (e *ErrorHandler) Errors() []error {
	return e.errs
}

func (e *ErrorHandler) Error() error {
	return errors.Join(e.errs...)
}

func (e *ErrorHandler) Pretty(line int, lineStr string, msg string, colStart int, colEnd int) {
	length := colEnd - colStart

	err := ""
	err += fmt.Sprintf("error: %s\n", msg)
	err += fmt.Sprintf("%3d | %s\n", line, lineStr)
	err += fmt.Sprintf("    | %s%s\n", strings.Repeat(" ", colStart), strings.Repeat("^", length))

	e.Add(fmt.Errorf("%s", err))
}
