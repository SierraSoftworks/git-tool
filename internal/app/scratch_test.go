package app_test

import (
	"fmt"
	"testing"
	"time"

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

func TestScratch(t *testing.T) {
	cmd := "scratch"

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

	reset := func() {
		out.Reset()
		launch.Reset()
		init.Reset()
	}

	year, week := time.Now().UTC().ISOWeek()
	yearweek := fmt.Sprintf("%dw%d", year, week)
	sp, err := di.GetMapper().GetScratchpad(yearweek)
	require.NoError(t, err, "we should be able to get the current scratchpad")
	require.NotNil(t, sp, "the current scratchpad should not be nil")

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")
			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CreateScratchpad",
				Target:   sp,
			}, "it should attempt to create the scratchpad")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, sp.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the scratchpad directory")
		}
	})

	t.Run("gt "+cmd+" app", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, "shell"), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")
			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CreateScratchpad",
				Target:   sp,
			}, "it should attempt to create the scratchpad")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, sp.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the scratchpad directory")
		}
	})

	t.Run("gt "+cmd+" new_sp", func(t *testing.T) {
		reset()

		if assert.NoError(t, runApp(cmd, sp.Name()), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")
			assert.Contains(t, init.MockCalls, struct {
				Function string
				Target   models.Target
			}{
				Function: "CreateScratchpad",
				Target:   sp,
			}, "it should attempt to create the scratchpad")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, sp.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the scratchpad directory")
		}
	})

	t.Run("gt "+cmd+" app existing_sp", func(t *testing.T) {
		reset()

		sp := repo.NewScratchpad("2019w15")
		if assert.NoError(t, runApp(cmd, "shell", sp.Name()), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")
			assert.Empty(t, init.MockCalls, "it should not attempt to re-create the scratchpad")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, sp.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the scratchpad directory")
		}
	})

	t.Run("gt "+cmd+" existing_sp", func(t *testing.T) {
		reset()

		sp := repo.NewScratchpad("2019w15")
		if assert.NoError(t, runApp(cmd, sp.Name()), "it should not return an error") {
			assert.Empty(t, out.GetOperations(), "it should not print any extra output")
			assert.Empty(t, init.MockCalls, "it should not attempt to re-create the scratchpad")

			assert.Len(t, launch.GetCommands(), 1, "it should have run a command")
			assert.Equal(t, "bash", launch.GetCommands()[0].Args[0], "it should have tried to launch the default app")
			assert.Equal(t, sp.Path(), launch.GetCommands()[0].Dir, "it should have tried to launch the app in the scratchpad directory")
		}
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

			assert.Contains(t, out.GetOperations(), "2019w15\n", "it should print a list of the scratchpads")
		})
	})
}
