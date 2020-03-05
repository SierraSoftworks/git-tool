package app_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConfigAdd(t *testing.T) {
	cmd := "config add"

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

	defer require.NoError(t, os.Chdir(test.GetProjectRoot()), "no error should be thrown when returning to the project root")
	defer require.NoError(t, os.RemoveAll(repo.Path()), "no error should be thrown when cleaning up the test repo")

	/*----- Tests -----*/

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		assert.Error(t, runApp("config", "add"), "it should return an error when called with no args")
		assert.Error(t, runApp(fmt.Sprintf("--config=%s", ""), "config", "add"), "it should return an error when there is no config file")
	})

	t.Run("gt "+cmd+" apps/bash", func(t *testing.T) {
		out.Reset()
		testConfig := test.GetTestDataPath("config.updated.yml")
		require.NoError(t, config.Default().Save(testConfig), "we should be able to generate a default config")
		defer os.RemoveAll(testConfig)

		assert.NoError(t, runApp("--config", testConfig, "config", "add", "apps/bash"), "it should not return an error")

		cfg, err := config.Load(testConfig)
		require.NoError(t, err, "we should be able to load the updated config")
		if assert.NotNil(t, cfg, "the config should not be nil") {
			assert.NotNil(t, cfg.GetApp("bash"), "it should have the new app in the config")
		}
	})

	t.Run("Auto Completion", func(t *testing.T) {
		t.Run("App-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", "gt config "), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "add\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "apps/\n", "it should include the apps/* prefix")
			assert.Contains(t, out.GetOperations(), "services/\n", "it should include the services/* prefix")
		})
	})
}
