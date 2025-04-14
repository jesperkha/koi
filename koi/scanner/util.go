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
