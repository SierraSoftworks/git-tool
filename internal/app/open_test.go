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
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestOpen(t *testing.T) {
	cmd := "open"

	/*----- Setup -----*/

	out := &mocks.Output{}
	init := &mocks.Initializer{}
	launch := &mocks.Launcher{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	di.SetInitializer(init)
	defer di.SetInitializer(&repo.Initializer{})
	di.SetLauncher(launch)
	defer di.SetLauncher(di.DefaultLauncher())

	existingRepo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
	require.NoError(t, err, "we should be able to get the test repo")
	require.NotNil(t, existingRepo, "we should be able to get the test repo")

	newRepo, err := di.GetMapper().GetRepo("github.com/git-fixtures/empty")
	require.NoError(t, err, "we should be able to get the test repo")
	require.NotNil(t, newRepo, "we should be able to get the test repo")

	reset := func() {
		out.Reset()
		launch.Reset()
		init.Reset()
	}

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		t.Run("Outside Repo", func(t *testing.T) {
			reset()

			assert.Error(t, runApp(cmd), "it should return a usage error")
			assert.Empty(t, out.GetOperations(), "it should not print any extra info")
		})

		t.Run("Inside Repo", func(t *testing.T) {
			reset()

			require.NoError(t, os.Chdir(existingRepo.Path()), "we should be able to switch to the repo")
			defer os.Chdir(test.GetProjectRoot())

			if assert.NoError(t, runApp(cmd), "it should not return an error") {
				assert.Empty(t, out.GetOperations(), "it should not print any extra info")
				assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
				assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
				assert.Equal(t, existingRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
			}
		})
	})

	t.Run("gt "+cmd+" app", func(t *testing.T) {
		t.Run("Outside Repo", func(t *testing.T) {
			reset()

			assert.Error(t, runApp(cmd, "shell"), "it should return a usage error")
			assert.Empty(t, out.GetOperations(), "it should not print any extra info")
		})

		t.Run("Inside Repo", func(t *testing.T) {
			reset()

			require.NoError(t, os.Chdir(existingRepo.Path()), "we should be able to switch to the repo")
			defer os.Chdir(test.GetProjectRoot())

			if assert.NoError(t, runApp(cmd, "shell"), "it should not return an error") {
				assert.Empty(t, out.GetOperations(), "it should not print any extra info")
				assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
				assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
				assert.Equal(t, existingRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
			}
		})
	})

	t.Run("gt "+cmd+" existing_repo", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, templates.RepoQualifiedName(existingRepo)), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")

			assert.Empty(t, init.MockCalls, "it should not attempt to clone the repo")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, existingRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
		}
	})

	t.Run("gt "+cmd+" new_repo", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, templates.RepoQualifiedName(newRepo)), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")

			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CloneRepository",
				Target:   newRepo,
			}, "it should attempt to clone the repository")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, newRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
		}
	})

	t.Run("gt "+cmd+" app existing_repo", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, "shell", templates.RepoQualifiedName(existingRepo)), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")

			assert.Empty(t, init.MockCalls, "it should not attempt to clone the repo")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, existingRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
		}
	})

	t.Run("gt "+cmd+" app new_repo", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, "shell", templates.RepoQualifiedName(newRepo)), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")

			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CloneRepository",
				Target:   newRepo,
			}, "it should attempt to clone the repository")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, newRepo.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the repo directory")
		}
	})

	t.Run("gt "+cmd+" bad_repo", func(t *testing.T) {
		reset()

		assert.Error(t, runApp(cmd, "bad_repo_name"), "it should return an error")
		assert.Empty(t, out.GetOperations(), "it should not print any extra output")

		assert.Empty(t, init.MockCalls, "it should not attempt to clone the repo")

		assert.Len(t, launch.GetCommands(), 0, "it should not attempt to run a command")
	})

	t.Run("Auto Completion", func(t *testing.T) {

		t.Run("App-Level", func(t *testing.T) {
			reset()
			require.NoError(t, runApp("complete", "gt"), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), cmd+"\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "github.com/sierrasoftworks/test1\n", "it should print a list of known repositories")
		})
	})
}
