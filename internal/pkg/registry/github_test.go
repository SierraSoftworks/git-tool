package registry_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGitHubSource(t *testing.T) {
	s := registry.GitHub()
	require.NotNil(t, s, "the registry should not be nil")

	t.Run("GetEntries()", func(t *testing.T) {
		entries, err := s.GetEntries()
		require.NoError(t, err, "there should not be an error getting the entries")

		assert.NotNil(t, entries, "the resulting entries list should not be nil")
	})
}
