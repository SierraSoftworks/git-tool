package gitignore_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGitIgnoreAPI(t *testing.T) {
	t.Run("List()", func(t *testing.T) {
		list, err := gitignore.List()

		require.NoError(t, err, "it should not return an error")
		assert.Greater(t, len(list), 1, "it should return a list with at least one item in it")

		for _, item := range list {
			assert.NotEmpty(t, item, "items should not be empty strings")
			assert.NotContains(t, item, ",", "items should not contain commas")
			assert.NotContains(t, item, "\n", "items should not contain newlines")
		}
	})

	t.Run("Ignore()", func(t *testing.T) {
		t.Run("Unrecognized Language", func(t *testing.T) {
			ignore, err := gitignore.Ignore("thisisnotareallanguage")
			assert.Error(t, err, "it should return an error")
			assert.NotEmpty(t, ignore, "it should not return an empty ignore file")
		})

		t.Run("Single Language", func(t *testing.T) {
			ignore, err := gitignore.Ignore("go")
			assert.NoError(t, err, "it should not return an error")
			assert.NotEmpty(t, ignore, "it should return a non-empty ignore file")
			assert.Contains(t, ignore, ".exe~", "it should return a valid ignore file")
		})

		t.Run("Multiple Languages", func(t *testing.T) {
			ignore, err := gitignore.Ignore("go", "node")
			assert.NoError(t, err, "it should not return an error")
			assert.NotEmpty(t, ignore, "it should return a non-empty ignore file")
			assert.Contains(t, ignore, ".exe~", "it should return a valid ignore file for the first language")
			assert.Contains(t, ignore, "node_modules", "it should return a valid ignore file for the second language")
		})
	})
}
