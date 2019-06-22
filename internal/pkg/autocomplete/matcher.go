package autocomplete

import "strings"

// Matches determines whether a value matches a specific ordered sequence of
// characters. For example "Albuquerque" shuould match "abq" but not "lab".
func Matches(value, sequence string) bool {
	if sequence == "" {
		return true
	}

	if len(value) < len(sequence) {
		return false
	}

	i := 0

	for _, c := range sequence {
		ii := strings.IndexRune(value[i:], c)
		if ii == -1 {
			return false
		}

		i = i + ii + 1
	}

	return true
}
