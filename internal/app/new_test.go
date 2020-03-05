package app_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNew(t *testing.T) {
	cmd := "new"

	/*----- Setup -----*/

	out := &mocks.Output{}
	init := &mocks.Initializer{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	di.SetInitializer(init)
	defer di.SetInitializer(&repo.Initializer{})

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		assert.Error(t, runApp(cmd), "it should return a usage error")
		assert.Empty(t, out.GetOperations(), "it should print no other output")
	})

	t.Run("gt "+cmd+"badreponame", func(t *testing.T) {
		out.Reset()
		assert.Error(t, runApp(cmd, "badreponame"), "it should return a usage error")
		assert.Empty(t, out.GetOperations(), "it should print no other output")
	})

	t.Run("gt "+cmd+"sierrasoftworks/test_new_repo", func(t *testing.T) {
		out.Reset()

		repo, err := di.GetMapper().GetBestRepo("sierrasoftworks/test_new_repo")
		require.NoError(t, err, "we should be able to get our repo")
		defer os.RemoveAll(repo.Path())

		assert.False(t, repo.Exists(), "the repo should not exist initially")

		if assert.NoError(t, runApp(cmd, repo.FullName()), "it should not return an error") {
			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CreateRepository",
				Target:   repo,
			}, "it should have called the CreateRepository initializer")
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

			assert.Contains(t, out.GetOperations(), "github.com/sierrasoftworks/\n", "it should print out a list of known prefixes")
		})
	})
}
