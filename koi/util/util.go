package util

// Returns position of last character on current line.
// Eg. the character right before newline or eof.
func FindEndOfLine(src []byte, offset int) int {
	for i := offset; i < len(src); i++ {
		c := src[i]
		if c == '\n' {
			return i - 1
		}
	}

	return len(src) - 1
}
