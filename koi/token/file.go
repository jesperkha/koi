package token

import (
	"fmt"
	"os"
)

type File struct {
	Name  string
	Src   []byte // File source
	Lines []int  // Offsets of beginning of each line, starting at 0.
	Err   error  // Error set on creation. Not returned by contructor for convenience.
}

func NewFile(filename string, src any) *File {
	file := &File{
		Name: filename,
	}

	srcBytes, err := readSource(filename, src)
	if err != nil {
		srcBytes = []byte{}
		file.Err = err
	}

	file.Src = srcBytes
	file.Lines = getLines(srcBytes)
	return file
}

func readSource(filename string, src any) ([]byte, error) {
	if src != nil {
		switch src := src.(type) {
		case string:
			return []byte(src), nil

		case []byte:
			return src, nil

		default:
			return nil, fmt.Errorf("invalid src type")
		}
	}

	return os.ReadFile(filename)
}

// Line returns the source at the given row (line number -1).
func (f *File) Line(row int) string {
	if row >= len(f.Lines) {
		panic("row out of bounds")
	}

	offset := f.Lines[row]
	end := findEndOfLine(f.Src, offset)
	return string(f.Src[offset:end])
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

func getLines(src []byte) []int {
	i := 0
	lines := []int{}
	for i < len(src) {
		lines = append(lines, i)
		i = findEndOfLine(src, i)
		i += 2 // Skip last char and newline
	}

	return lines
}
