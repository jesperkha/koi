package token

import "testing"

func TestFindEndOfLine(t *testing.T) {
	src := []byte("hello there\nmy name is bob")

	// offset: expect
	cases := map[int]int{
		5:  10,
		14: 25,
	}

	for k, v := range cases {
		if n := findEndOfLine(src, k); n != v {
			t.Errorf("expected end=%d, got %d, for offset=%d", v, n, k)
		}
	}
}
