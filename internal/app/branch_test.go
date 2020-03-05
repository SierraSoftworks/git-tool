package app_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBranch(t *testing.T) {
	cmd := "branch"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	repo, err := di.GetMapper().GetFullyQualifiedRepo("github.com/sierrasoftworks/branch_test_repo")
	require.NoError(t, err, "no error should be thrown when creating the test repo")
	require.NoError(t, tasks.Sequence(
		tasks.NewFolder(),
		tasks.GitInit(),
		tasks.NewFile("README.md", []byte("# Test Repo")),
		tasks.GitCommit("Initial Commit", "README.md"),
		tasks.GitCheckout("master", false),
	).ApplyRepo(repo), "no error should be thrown when initializing the test repo")
	require.NoError(t, os.Chdir(repo.Path()), "no error should be thrown when cd-ing into the repo")

	defer func() {
		os.Chdir(test.GetProjectRoot())
		os.RemoveAll(repo.Path())
	}()

	cwd, err := os.Getwd()
	require.NoError(t, err, "there should not be an error getting the current working directory")
	require.Equal(t, repo.Path(), cwd, "the current directory should be in the test repo")

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		assert.Error(t, runApp(cmd), "it should return an error when called with no args")
	})

	t.Run("Auto Completion", func(t *testing.T) {
		t.Run("App-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", "gt"), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), cmd+"\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "master\n", "it should print out the list of branch names")
		})
	})
}
