package autocomplete_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/stretchr/testify/assert"
)

func TestInitScripts(t *testing.T) {
	cases := []struct {
		Shell    string
		HasValue bool
	}{
		{"powershell", true},
		{"bash", true},
		{"zsh", true},
		{"cmd", false},
		{"fish", false},
	}

	for _, tc := range cases {
		if tc.HasValue {
			assert.NotEmptyf(t, autocomplete.GetInitScript(tc.Shell), "GetInitScript('%s') should return the init script", tc.Shell)
			assert.NotEmptyf(t, autocomplete.GetFullInitScript(tc.Shell), "GetFullInitScript('%s') should return the init script", tc.Shell)
			assert.Containsf(t, autocomplete.GetInitScriptShells(), tc.Shell, "GetInitScriptShells() should contain '%s'", tc.Shell)
		} else {
			assert.Emptyf(t, autocomplete.GetInitScript(tc.Shell), "GetInitScript('%s') should not return an init script", tc.Shell)
			assert.Emptyf(t, autocomplete.GetFullInitScript(tc.Shell), "GetFullInitScript('%s') should not return an init script", tc.Shell)
			assert.NotContainsf(t, autocomplete.GetInitScriptShells(), tc.Shell, "GetInitScriptShells() should not contain '%s'", tc.Shell)
		}
	}
}
