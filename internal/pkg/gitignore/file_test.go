package gitignore_test

import (
	"io/ioutil"
	"os"
	"strings"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAddOrUpdate(t *testing.T) {
	t.Run("New File", func(t *testing.T) {
		filePath := test.GetTestDataPath("ignore", ".gitignore")

		fileContent := func(t *testing.T) string {
			c, err := ioutil.ReadFile(filePath)
			if os.IsNotExist(err) {
				return ""
			}

			require.NoError(t, err, "the test file should be readable")
			return string(c)
		}

		t.Run("No languages", func(t *testing.T) {
			defer os.RemoveAll(filePath)

			assert.Empty(t, fileContent(t), "the original file should not exist")
			assert.NoError(t, gitignore.AddOrUpdate(filePath), "it should not throw an error")
			assert.Empty(t, fileContent(t), "it should not write any new content to the file")
		})

		t.Run("Invalid language", func(t *testing.T) {
			defer os.RemoveAll(filePath)

			assert.Empty(t, fileContent(t), "the original file should not exist")
			assert.Error(t, gitignore.AddOrUpdate(filePath, "thisisnotareallanguage"), "it should throw an error")
			assert.Empty(t, fileContent(t), "it should not write any content to the file")
		})

		t.Run("New language", func(t *testing.T) {
			defer os.RemoveAll(filePath)

			assert.Empty(t, fileContent(t), "the original file should not exist")
			assert.NoError(t, gitignore.AddOrUpdate(filePath, "go"), "it should not throw an error")
			assert.NotEmpty(t, fileContent(t), "it should write the language's ignore file")
		})
	})

	t.Run("Old File", func(t *testing.T) {
		filePath := test.GetTestDataPath("ignore", "oldgo.gitignore")

		fileContent := func(t *testing.T) string {
			c, err := ioutil.ReadFile(filePath)
			require.NoError(t, err, "the test file should be readable")
			return string(c)
		}

		originalContent := strings.TrimSpace(`
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go

*.exe
*.exe~
*.dll

# End of https://www.gitignore.io/api/go		
`)

		t.Run("No languages", func(t *testing.T) {
			require.NoError(t, ioutil.WriteFile(filePath, []byte(originalContent), os.ModePerm), "the test file should be restored to the original state")

			assert.NoError(t, gitignore.AddOrUpdate(filePath), "it should not return any errors")
			newContent := fileContent(t)

			assert.NotEqual(t, originalContent, newContent, "the file should be updated to the latest version")
		})

		t.Run("Same languages", func(t *testing.T) {
			require.NoError(t, ioutil.WriteFile(filePath, []byte(originalContent), os.ModePerm), "the test file should be restored to the original state")

			assert.NoError(t, gitignore.AddOrUpdate(filePath, "go"), "it should not return any errors")
			newContent := fileContent(t)

			assert.NotEqual(t, originalContent, newContent, "the file should be updated to the latest version")
		})

		t.Run("New languages", func(t *testing.T) {
			require.NoError(t, ioutil.WriteFile(filePath, []byte(originalContent), os.ModePerm), "the test file should be restored to the original state")

			assert.NoError(t, gitignore.AddOrUpdate(filePath, "node"), "it should not return any errors")
			newContent := fileContent(t)

			assert.NotEqual(t, originalContent, newContent, "the file should be updated with the new languages")
		})

		require.NoError(t, ioutil.WriteFile(filePath, []byte(originalContent), os.ModePerm), "the test file should be restored to the original state")
	})
}
