package app_test

import (
	"fmt"
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

func TestList(t *testing.T) {
	cmd := "list"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd), "it should not return an error") {
			repos, err := di.GetMapper().GetRepos()
			require.NoError(t, err, "we should be able to enumerate the repos")

			assert.Len(t, out.GetOperations(), len(repos), "it should print out the list of repos")

			for _, repo := range repos {
				assert.Contains(t, out.GetOperations(), templates.RepoShortInfo(repo)+"\n", "it should print out the repo name")
			}
		}
	})

	t.Run("gt "+cmd+" --quiet", func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd, "--quiet"), "it should not return an error") {
			repos, err := di.GetMapper().GetRepos()
			require.NoError(t, err, "we should be able to enumerate the repos")

			assert.Len(t, out.GetOperations(), len(repos), "it should print out the list of repos")

			for _, repo := range repos {
				assert.Contains(t, out.GetOperations(), templates.RepoQualifiedName(repo)+"\n", "it should print out the repo name")
			}
		}
	})

	t.Run("gt "+cmd+" --full", func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd, "--full"), "it should not return an error") {
			repos, err := di.GetMapper().GetRepos()
			require.NoError(t, err, "we should be able to enumerate the repos")

			for _, repo := range repos {
				assert.Contains(t, out.GetOperations(), templates.RepoFullInfo(repo)+"\n", "it should print out the repo name")
			}
		}
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

			assert.Empty(t, out.GetOperations(), "it should not print any completion suggestions")
		})
	})
}
