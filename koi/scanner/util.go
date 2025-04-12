package scanner

import "strings"

func isAlpha(c byte) bool {
	return strings.Contains("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_", string(c))
}

func isNum(c byte) bool {
	return strings.Contains("0123456789", string(c))
}

func isWhitespace(c byte) bool {
	return strings.Contains("\n\t\r ", string(c))
}

// Returns position of last character on current line.
// Eg. the character right before newline or eof.
func findEndOfLine(src []byte, offset int) int {
	for i := offset; i < len(src); i++ {
		c := src[i]
		if c == '\n' {
			return i - 1
		}
	}

	return len(src) - 1
}
