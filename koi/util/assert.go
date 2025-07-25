package util

import "fmt"

func Assert(v bool, format string, args ...any) {
	if !v {
		panic(fmt.Sprintf("assertion failed: %s", fmt.Sprintf(format, args...)))
	}
}
