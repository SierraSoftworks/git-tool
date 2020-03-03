package autocomplete_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/stretchr/testify/assert"
)

func TestMatcher(t *testing.T) {
	cases := []struct {
		Value string
		Seq   string
		Match bool
	}{
		{"test", "t", true},
		{"test", "x", false},
		{"test", "te", true},
		{"test", "tst", true},
		{"test", "tsts", false},
		{"test", "et", true},
	}

	for _, tc := range cases {
		assert.Equalf(t, tc.Match, autocomplete.Matches(tc.Value, tc.Seq), "Matches('%s', '%s') should be %v", tc.Value, tc.Seq, tc.Match)
	}
}
