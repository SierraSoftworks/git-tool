package registry_test

import (
	"path/filepath"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFilesystemSource(t *testing.T) {
	s := registry.FileSystem(filepath.Join(test.GetProjectRoot(), "registry"))
	require.NotNil(t, s, "the registry should not be nil")

	t.Run("GetEntries()", func(t *testing.T) {
		entries, err := s.GetEntries()
		require.NoError(t, err, "there should not be an error getting the entries")

		require.NotEmpty(t, entries, "there should be at least one entry")
		assert.Contains(t, entries, "apps/bash", "it should contain the bash entry")
	})

	t.Run("GetEntry(id)", func(t *testing.T) {
		entry, err := s.GetEntry("apps/bash")
		require.NoError(t, err, "there should not be an error getting an entry")

		require.NotNil(t, entry, "the resulting entry should not be nil")
		assert.Equal(t, entry.Name, "Bash", "the resulting entry should have the right name")
	})
}
