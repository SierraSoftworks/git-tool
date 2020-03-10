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
	"gopkg.in/src-d/go-git.v4"
)

func TestGitCheckout(t *testing.T) {
	cfg := mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetLauncher(di.DefaultLauncher())
	di.SetMapper(&repo.Mapper{})
	di.SetInitializer(&repo.Initializer{})
	di.SetConfig(cfg)

	t.Run("Repo", func(t *testing.T) {

		t.Run("Missing", func(t *testing.T) {
			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
			defer os.RemoveAll(r.Path())
			out.Reset()

			assert.Error(t, tasks.GitCheckout("master", false).ApplyRepo(r), "it should return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.False(t, r.Exists(), "it should not create the repository folder")
		})

		t.Run("Uninitialized", func(t *testing.T) {
			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
			defer os.RemoveAll(filepath.Join(r.Path(), ".git"))
			out.Reset()

			assert.Error(t, tasks.GitCheckout("master", false).ApplyRepo(r), "it should return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, r.Exists(), "the repository folder should still exist")
			assert.False(t, r.Valid(), "it should not initialize the repository folder")
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
				tasks.GitNewRef("refs/remotes/origin/test-branch"),
				tasks.GitNewRef("refs/remotes/origin/test-branch2"),
				tasks.NewFile("README.md", []byte("# Test Repo\nWith changes")),
				tasks.GitCommit("Made changes to README", "README.md"),
				tasks.GitNewRef("refs/heads/test-branch2"),
				tasks.GitCheckout("master", false),
			).ApplyRepo(r), "the repository should be setup correctly for the test")

			gr, err := git.PlainOpen(r.Path())
			require.NoError(t, err, "we should be able to read the git repo")

			master, err := gr.Reference("refs/heads/master", true)
			require.NoError(t, err, "we should be able to get the 'master' ref of the repo")

			testBranch, err := gr.Reference("refs/remotes/origin/test-branch", true)
			require.NoError(t, err, "we should be able to get the 'test-branch' ref of the repo")

			t.Run("Local Branch", func(t *testing.T) {
				out.Reset()

				require.NoError(t, tasks.GitCheckout("master", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, master.Hash().String(), head.Hash().String(), "it should have the correct branch checked out")
			})

			t.Run("Remote Branch", func(t *testing.T) {
				out.Reset()

				require.NoError(t, tasks.GitCheckout("test-branch", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, testBranch.Hash().String(), head.Hash().String(), "it should have the correct branch checked out")
			})

			t.Run("Explicit Remote Branch", func(t *testing.T) {
				out.Reset()

				require.NoError(t, tasks.GitCheckout("origin/test-branch", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, testBranch.Hash().String(), head.Hash().String(), "it should have the correct branch checked out")
			})

			t.Run("Local and Remote Branch", func(t *testing.T) {
				out.Reset()

				require.NoError(t, tasks.GitCheckout("test-branch2", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, master.Hash().String(), head.Hash().String(), "it should have the correct branch checked out")
			})

			t.Run("Explicit Remote Branch with Local Copy", func(t *testing.T) {
				out.Reset()

				require.NoError(t, tasks.GitCheckout("origin/test-branch2", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, testBranch.Hash().String(), head.Hash().String(), "it should have the correct branch checked out")
			})

			t.Run("New Branch", func(t *testing.T) {
				out.Reset()

				oldHead, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")

				require.NoError(t, tasks.GitCheckout("test", false).ApplyRepo(r), "it should not return any errors")
				assert.Empty(t, out.GetOperations(), "it should not log anything")
				assert.True(t, r.Exists(), "the repository folder should still exist")

				head, err := gr.Head()
				require.NoError(t, err, "we should be able to get the HEAD hash of the repo")
				assert.Equal(t, oldHead.Hash().String(), head.Hash().String(), "it should have created the branch at the original HEAD")
			})
		})
	})

	t.Run("Scratchpad", func(t *testing.T) {
		sp := repo.NewScratchpad("2019w28")
		defer os.RemoveAll(sp.Path())
		out.Reset()

		require.NoError(t, tasks.GitCheckout("master", true).ApplyScratchpad(sp), "it should not return an error")
		assert.Empty(t, out.GetOperations(), "it should not log anything")
		assert.False(t, sp.Exists(), "it should not have created the scratchpad folder")
	})
}
