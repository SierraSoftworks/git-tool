package app_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInfo(t *testing.T) {
	cmd := "info"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
	require.NoError(t, err, "we should be able to get the test repo")
	require.NotNil(t, repo, "we should be able to get the test repo")

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		t.Run("Outside Repo", func(t *testing.T) {
			out.Reset()
			assert.Error(t, runApp(cmd), "it should return an error if you're not in the repo's directory")
		})

		t.Run("Inside Repo", func(t *testing.T) {
			out.Reset()
			require.NoError(t, os.Chdir(repo.Path()), "we should be able to switch to the repo")
			defer os.Chdir(test.GetProjectRoot())

			if assert.NoError(t, runApp(cmd), "it should not return an error") {
				assert.Contains(t, out.GetOperations(), templates.RepoFullInfo(repo)+"\n", "it should print the repo's full info")
			}
		})
	})

	t.Run("gt "+cmd+" existing_repo", func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd, templates.RepoQualifiedName(repo)), "it should not return an error") {
			assert.Contains(t, out.GetOperations(), templates.RepoFullInfo(repo)+"\n", "it should print the repo's full info")
		}
	})

	t.Run("gt "+cmd+" invalid_repo", func(t *testing.T) {
		out.Reset()
		assert.Error(t, runApp(cmd, "invalidreponame"), "it should return an error")

		assert.Empty(t, out.GetOperations(), "it should not print any other output")
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

			assert.Contains(t, out.GetOperations(), "github.com/sierrasoftworks/test1\n", "it should print a list of repos")
		})
	})
}
