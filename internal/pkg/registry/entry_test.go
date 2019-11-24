package registry_test

import (
	"runtime"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/stretchr/testify/assert"
)

func TestEntryConfig(t *testing.T) {
	t.Run("IsCompatible()", func(t *testing.T) {

		cases := []struct {
			Platform string
			Expected bool
		}{
			{"any", true},
			{"windows", runtime.GOOS == "windows"},
			{"linux", runtime.GOOS == "linux"},
			{"darwin", runtime.GOOS == "darwin"},
		}

		for _, tc := range cases {
			entry := &registry.EntryConfig{
				Platform: tc.Platform,
			}

			assert.Equal(t, tc.Expected, entry.IsCompatible())
		}
	})
}
