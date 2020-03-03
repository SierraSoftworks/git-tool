package gitignore_test

import (
	"strings"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFileHeader(t *testing.T) {
	t.Run("At the start of a file", func(t *testing.T) {
		content := strings.TrimSpace(`
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go,rust, csharp
*.exe`)

		header := gitignore.ParseSection(content)
		require.NotNil(t, header, "it should not return a nil header")

		assert.Equal(t, header.Languages, []string{"go", "rust", "csharp"}, "it should have the correct languages")
		assert.Equal(t, header.Prologue, "", "it should have the correct prologue")
		assert.Equal(t, header.Content, "*.exe", "it should have the correct content")
	})

	t.Run("At the end of a file", func(t *testing.T) {
		content := strings.TrimSpace(`
junit.xml
bin/

## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: csharp, java
*.exe
*.obj`)

		header := gitignore.ParseSection(content)
		require.NotNil(t, header, "it should not return a nil header")

		assert.Equal(t, header.Languages, []string{"csharp", "java"}, "it should have the correct languages")
		assert.Equal(t, header.Prologue, "junit.xml\nbin/\n", "it should have the correct prologue")
		assert.Equal(t, header.Content, "*.exe\n*.obj", "it should have the correct content")
	})
}
