package tasks_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewFolder(t *testing.T) {
	cfg := mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetLauncher(di.DefaultLauncher())
	di.SetMapper(&repo.Mapper{})
	di.SetInitializer(&repo.Initializer{})
	di.SetConfig(cfg)

	task := tasks.NewFolder()

	t.Run("Repo", func(t *testing.T) {
		t.Run("Missing", func(t *testing.T) {
			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
			defer os.RemoveAll(r.Path())
			out.Reset()

			assert.NoError(t, task.ApplyRepo(r), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, r.Exists(), "it should have created the repository folder")
			assert.False(t, r.Valid(), "it should not have initialized the repository")
		})

		t.Run("Uninitialized", func(t *testing.T) {
			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
			defer os.RemoveAll(filepath.Join(r.Path(), ".git"))
			out.Reset()

			assert.NoError(t, task.ApplyRepo(r), "it should return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, r.Exists(), "the repository folder should still exist")
			assert.False(t, r.Valid(), "it should not have initialized the repository")
		})

		t.Run("Valid Repo", func(t *testing.T) {
			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test4")
			defer os.RemoveAll(r.Path())

			require.NoError(t, tasks.Sequence(
				tasks.NewFolder(),
				tasks.GitInit(),
				tasks.GitRemote("origin"),
				tasks.NewFile("README.md", []byte("# Test Repo")),
				tasks.GitCommit("Initial Commit", "README.md"),
				tasks.GitCheckout("main", false),
			).ApplyRepo(r), "the repository should be setup correctly for the test")

			assert.NoError(t, task.ApplyRepo(r), "it should return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, r.Exists(), "the repository folder should still exist")
			assert.True(t, r.Valid(), "the repository should still be valid")
		})
	})

	t.Run("Scratchpad", func(t *testing.T) {
		t.Run("Missing", func(t *testing.T) {
			sp := repo.NewScratchpad("2019w28")
			defer os.RemoveAll(sp.Path())
			out.Reset()

			require.NoError(t, task.ApplyScratchpad(sp), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, sp.Exists(), "it should have created the scratchpad folder")
		})

		t.Run("Existing", func(t *testing.T) {
			sp := repo.NewScratchpad("2019w27")
			out.Reset()

			require.NoError(t, task.ApplyScratchpad(sp), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, sp.Exists(), "the scratchpad folder should still exist")
		})
	})
}
